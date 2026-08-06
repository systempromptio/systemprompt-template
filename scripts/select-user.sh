#!/usr/bin/env bash
#
# select-user.sh - pick the database user a demo should act as.
#
# Source this; do not execute it:
#   CLI=... PROFILE=... source "$PROJECT_DIR/scripts/select-user.sh"
#
# The demos used to invent their identity (pi-demo@demo.local,
# admin@localhost.dev), so every session, request, cost row, and governance
# decision they produced landed on a synthetic account instead of a real
# profile page. These helpers list the users that already exist and mint
# credentials for the one the operator picks.
#
#   select_db_user [--admins-only] [preselect-email]
#       Sets SEL_USER_ID / SEL_USER_EMAIL / SEL_USER_NAME / SEL_USER_ROLES /
#       SEL_USER_IS_ADMIN / SEL_USER_CREATED_THIS_RUN.
#
#   ensure_plugin_token <user-id> <email>
#       Mints a plugin JWT without ever mutating a pre-existing user's roles.
#       Echoes the token on stdout.
#
# Callers must set $CLI (systemprompt binary) and $PROFILE first.
# macOS + Linux safe: no `grep -P`, no `head -n -1`, no GNU-only sed features.

# jq drives every listing below; the CLI only emits machine-readable JSON under
# the global --json flag, and parsing the human box-table instead is how demos
# start narrating fiction.
if ! command -v jq >/dev/null 2>&1; then
  echo "ERROR: jq is required (brew install jq / apt-get install jq)" >&2
  return 1 2>/dev/null || exit 1
fi

# Rows for one role as TSV: id, email, name, roles. `--status active` keeps
# deleted and suspended accounts out of the menu — `--role` matches on role
# alone, so without it a deleted admin is offered as a valid pick.
#
# Non-UUID ids are dropped, because core parses the id as a UUID on every path
# that authenticates (oauth providers, the JWT authn/authz middleware, the
# OAuth callback). An id that is not one cannot sign in and cannot have a token
# minted for it, so offering it as someone to act as only ever ends in
# "is not a valid UUID" further down.
#
# 027 gave the demo-organizations fixtures real UUIDs, which was the case that
# used to take the whole suite down at preflight. This stays as the general
# guard: the `system` account still carries a non-UUID id and the admin role,
# and nobody should be acting as that either.
_sel_list_role() { # $1=role
  "$CLI" --json admin users list --role "$1" --status active --limit 200 --profile "$PROFILE" 2>/dev/null \
    | jq -r '.items[]?
        | select(.id | test("^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"))
        | [.id, .email, (.name // .display_name // ""), ((.roles // []) | join(","))] | @tsv' \
    || true
}

# Admins first: they are the likely pick, and --admins-only reuses this. A user
# holding both roles comes back from both queries, so dedupe on id — first
# occurrence wins, which keeps them in the admin block.
_sel_all_rows() { # $1=admins_only
  {
    _sel_list_role admin
    [[ "$1" == "1" ]] || _sel_list_role user
  } | awk -F'\t' 'NF && !seen[$1]++'
}

_sel_assign() { # $1=tsv row
  SEL_USER_ID=$(printf '%s' "$1" | cut -f1)
  SEL_USER_EMAIL=$(printf '%s' "$1" | cut -f2)
  SEL_USER_NAME=$(printf '%s' "$1" | cut -f3)
  SEL_USER_ROLES=$(printf '%s' "$1" | cut -f4)
  if [[ ",$SEL_USER_ROLES," == *",admin,"* ]]; then
    SEL_USER_IS_ADMIN=1
  else
    SEL_USER_IS_ADMIN=0
  fi
}

# Create (or adopt) a demo-owned account. SEL_USER_CREATED_THIS_RUN marks it as
# fair game for the promote/mint/demote dance in ensure_plugin_token; a user
# picked from the menu never carries the flag, so their roles stay untouched.
_sel_create_user() { # $1=email $2=name
  "$CLI" admin users create --name "$2" --email "$1" --if-not-exists --profile "$PROFILE" 2>&1 \
    | grep -viE '^\[profile|already exists' || true
  local row
  row=$("$CLI" --json admin users search "$1" --profile "$PROFILE" 2>/dev/null \
    | jq -r '.items[0]? | [.id, .email, (.name // .display_name // ""), ((.roles // []) | join(","))] | @tsv')
  if [[ -z "$row" || "$row" == "null" ]]; then
    echo "ERROR: could not locate $1 after creation." >&2
    return 1
  fi
  _sel_assign "$row"
  SEL_USER_CREATED_THIS_RUN=1
}

select_db_user() {
  local admins_only=0
  if [[ "${1:-}" == "--admins-only" ]]; then
    admins_only=1
    shift
  fi
  local preselect="${1:-}"

  SEL_USER_ID=""; SEL_USER_EMAIL=""; SEL_USER_NAME=""
  SEL_USER_ROLES=""; SEL_USER_IS_ADMIN=0; SEL_USER_CREATED_THIS_RUN=0

  local rows
  rows=$(_sel_all_rows "$admins_only")

  # Resolution order: explicit argument > DEMO_SELECT_EMAIL > the legacy
  # ADMIN_EMAIL / DEMO_USER_EMAIL overrides the scenario scripts already set.
  local wanted="${preselect:-${DEMO_SELECT_EMAIL:-}}"
  if [[ -z "$wanted" && $admins_only -eq 1 ]]; then
    wanted="${ADMIN_EMAIL:-}"
  elif [[ -z "$wanted" ]]; then
    wanted="${DEMO_USER_EMAIL:-}"
  fi

  if [[ -n "$wanted" ]]; then
    local match
    match=$(printf '%s\n' "$rows" | awk -F'\t' -v e="$wanted" '$2 == e {print; exit}')
    if [[ -n "$match" ]]; then
      _sel_assign "$match"
      return 0
    fi
    # A named email that matches nothing is a typo or the wrong profile, not an
    # invitation to silently create a second account under a similar address.
    echo "ERROR: no user '$wanted' in the ${PROFILE} database." >&2
    echo "  Listed users:" >&2
    printf '%s\n' "$rows" | awk -F'\t' 'NF {print "    " $2 "  (" $4 ")"}' >&2
    return 1
  fi

  if [[ -z "$rows" ]]; then
    # An empty admin list is a first-run database; the caller decides whether
    # to bootstrap one, so report it rather than guessing.
    return 2
  fi

  # No tty (CI, a piped run) means nobody can answer a menu: take the first
  # row, which is an admin whenever the database has one.
  if [[ ! -t 0 ]]; then
    _sel_assign "$(printf '%s\n' "$rows" | awk 'NF {print; exit}')"
    return 0
  fi

  echo "" >&2
  echo "  Select the user to act as:" >&2
  echo "" >&2
  local i=0 line
  local -a table=()
  while IFS= read -r line; do
    [[ -n "$line" ]] || continue
    i=$((i + 1))
    table+=("$line")
    local email name roles tag
    email=$(printf '%s' "$line" | cut -f2)
    name=$(printf '%s' "$line" | cut -f3)
    roles=$(printf '%s' "$line" | cut -f4)
    tag=""
    [[ ",$roles," == *",admin,"* ]] && tag=" [admin]"
    printf '    %2d) %s%s  %s  (%s)\n' "$i" "$email" "$tag" "${name:-—}" "$roles" >&2
  done <<< "$rows"
  echo "     n) create a new demo user" >&2
  echo "" >&2

  local reply=""
  printf '  Choice [1]: ' >&2
  read -r reply || true
  reply="${reply:-1}"

  if [[ "$reply" == "n" || "$reply" == "N" ]]; then
    local new_email new_name
    printf '  Email for the new user [pi-demo@demo.local]: ' >&2
    read -r new_email || true
    new_email="${new_email:-pi-demo@demo.local}"
    printf '  Display name [Pi Demo]: ' >&2
    read -r new_name || true
    new_name="${new_name:-Pi Demo}"
    case "$new_email" in
      *@*.*) ;;
      *) echo "ERROR: '$new_email' does not look like an email address" >&2; return 1 ;;
    esac
    _sel_create_user "$new_email" "$new_name"
    return $?
  fi

  if ! [[ "$reply" =~ ^[0-9]+$ ]] || (( reply < 1 || reply > ${#table[@]} )); then
    echo "ERROR: '$reply' is not one of the choices above." >&2
    return 1
  fi
  _sel_assign "${table[$((reply - 1))]}"
}

# Mint a plugin JWT for a user without ever demoting one we did not create.
#
# `admin keys issue-plugin-token` refuses non-admins, and governance resolves
# access scope from the caller's LIVE DB role rather than from the token — which
# is why the demos historically promoted, minted, and demoted. Run against a
# real admin that silently strips their admin role and breaks every token-based
# demo, so the dance is now confined to users created in this run.
ensure_plugin_token() { # $1=user-id $2=email
  local user_id="$1" email="$2" token=""

  # A demo-owned account is checked first: it must end the call with role `user`
  # whatever it started as, or the deny demos that rely on it fail open.
  if [[ "${SEL_USER_CREATED_THIS_RUN:-0}" == "1" ]]; then
    "$CLI" admin users role promote "$user_id" --profile "$PROFILE" >/dev/null 2>&1 || true
    token=$("$CLI" admin keys issue-plugin-token --token-only \
      --email "$email" --profile "$PROFILE" 2>/dev/null \
      | grep -oE 'eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+' | head -1 || true)
    "$CLI" admin users role demote "$user_id" --profile "$PROFILE" >/dev/null 2>&1 || true
    SEL_USER_IS_ADMIN=0
  elif [[ "${SEL_USER_IS_ADMIN:-0}" == "1" ]]; then
    token=$("$CLI" admin keys issue-plugin-token --token-only \
      --email "$email" --profile "$PROFILE" 2>/dev/null \
      | grep -oE 'eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+' | head -1 || true)
  else
    echo "ERROR: $email is not an admin, so a plugin token cannot be minted" >&2
    echo "  without a temporary role change — and this user already existed," >&2
    echo "  so the demo will not touch their roles. Do it yourself if intended:" >&2
    echo "" >&2
    echo "    $CLI admin users role promote $user_id --profile $PROFILE" >&2
    echo "    # re-run this script, then:" >&2
    echo "    $CLI admin users role demote $user_id --profile $PROFILE" >&2
    return 1
  fi

  if [[ -z "$token" ]]; then
    echo "ERROR: could not mint a plugin token for $email." >&2
    return 1
  fi
  printf '%s' "$token"
}
