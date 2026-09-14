#!/usr/bin/env bash
# Clean-client publication/install/rollback acceptance. The caller supplies an
# authenticated PAT file; credentials are never placed in argv or Docker env.
set -euo pipefail

: "${GATEWAY:?GATEWAY is required}"
: "${ORIGIN:?ORIGIN is required}"
: "${PAT_FILE:?PAT_FILE is required}"
: "${OWNER_ID:?OWNER_ID is required}"
: "${RESOURCE_ID:?RESOURCE_ID is required}"
: "${BASELINE_REVISION_ID:?BASELINE_REVISION_ID is required}"
: "${CANDIDATE_REVISION_ID:?CANDIDATE_REVISION_ID is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"

mkdir -p "$EVIDENCE_DIR" /tmp/systemprompt-install
token="$(<"$PAT_FILE")"
test -n "$token"

api() {
    method="$1"
    path="$2"
    body="${3:-}"
    args=(-fsS -X "$method" -H "Authorization: Bearer $token" -H "Origin: $ORIGIN")
    if [ -n "$body" ]; then args+=(-H 'Content-Type: application/json' --data-binary "@$body"); fi
    curl "${args[@]}" "$GATEWAY/api/public/admin$path"
}

publish() {
    action="$1" revision="$2" expected="$3" key="$4" evidence="$5" output="$6"
    jq -n --arg resource "$RESOURCE_ID" --arg revision "$revision" --arg action "$action" \
        --arg key "$key" --argjson expected "$expected" --argjson evidence "$evidence" \
        '{resource_id:$resource,revision_id:$revision,action:$action,expected_generation:$expected,operation_key:$key,comparison_evidence:$evidence,limitations:"Bounded clean-client acceptance"}' > /tmp/publication.json
    api POST /managed/publications /tmp/publication.json | tee "$output"
}

distribute() {
    generation="$1" output="$2"
    claim_token="clean-lifecycle-$generation-$RESOURCE_ID"
    jq -n --arg token "$claim_token" '{claim_token:$token}' > /tmp/claim.json
    api POST /managed/distributions/claim /tmp/claim.json | tee "$output"
    jq -e '. != null' "$output" >/dev/null
    jq '{claim:.,delivered:true,error:null}' "$output" > /tmp/complete.json
    api POST /managed/distributions/complete /tmp/complete.json >/dev/null
}

install_and_receipt() {
    publication_file="$1" label="$2"
    publication_id="$(jq -er '.publication_id' "$publication_file")"
    generation="$(jq -er '.generation' "$publication_file")"
    digest="$(jq -er '.bundle_digest' "$publication_file")"
    destination="/tmp/systemprompt-install/$label"
    mkdir -p "$destination"
    bundle="$EVIDENCE_DIR/$label-bundle.json"
    api GET "/managed/resources/$RESOURCE_ID/publications/$generation/bundle?digest=$digest" > "$bundle"
    test "$(sha256sum "$bundle" | cut -d' ' -f1)" = "$digest"

    manifest="$EVIDENCE_DIR/$label-installed-files.jsonl"
    : > "$manifest"
    jq -c '. as $bundle | .revisions | to_entries[] as $revision | $revision.value.files | to_entries[] | {revision_id:$revision.key,path:.key,digest:.value.digest,bytes:.value.bytes,executable:.value.executable,content:$bundle.assets[.value.digest]}' "$bundle" |
    while IFS= read -r entry; do
        path="$(jq -er '.path' <<<"$entry")"
        case "/$path/" in *'/../'*|*'/./'*|*'//'*) echo "unsafe bundle path: $path" >&2; exit 1;; esac
        target="$destination/$path"
        mkdir -p "$(dirname "$target")"
        jq -c '.content' <<<"$entry" | python3 -c 'import json,sys; sys.stdout.buffer.write(bytes(json.load(sys.stdin)))' > "$target"
        if jq -e '.executable' <<<"$entry" >/dev/null; then chmod 700 "$target"; expected_mode=700; else chmod 600 "$target"; expected_mode=600; fi
        test "$(sha256sum "$target" | cut -d' ' -f1)" = "$(jq -er '.digest' <<<"$entry")"
        test "$(wc -c < "$target")" = "$(jq -er '.bytes' <<<"$entry")"
        test "$(stat -c '%a' "$target")" = "$expected_mode"
        jq -c 'del(.content)' <<<"$entry" >> "$manifest"
    done
    jq -s '.' "$manifest" > /tmp/files.json
    jq -n --arg installation "clean-client-$label" --arg publication "$publication_id" --arg resource "$RESOURCE_ID" \
        --argjson generation "$generation" --arg digest "$digest" --arg owner "$OWNER_ID" --arg session "clean-lifecycle-$label" \
        --slurpfile files /tmp/files.json '{installation_id:$installation,publication_id:$publication,resource_id:$resource,generation:$generation,bundle_digest:$digest,files:$files[0],client_evidence:{owner_id:$owner,session_id:$session,client:"clean-client",verification:"files-digests-lengths-modes"}}' > /tmp/receipt.json
    api POST /managed/installations /tmp/receipt.json | tee "$EVIDENCE_DIR/$label-receipt.json"
}

baseline="$EVIDENCE_DIR/baseline-publication.json"
candidate="$EVIDENCE_DIR/candidate-publication.json"
rollback="$EVIDENCE_DIR/rollback-publication.json"
publish initial_adoption "$BASELINE_REVISION_ID" 0 clean-baseline '{}' "$baseline"
distribute 1 "$EVIDENCE_DIR/baseline-distribution.json"
install_and_receipt "$baseline" baseline
publish publish_improvement "$CANDIDATE_REVISION_ID" 1 clean-candidate '{"acceptance":"candidate reviewed by harness"}' "$candidate"
distribute 2 "$EVIDENCE_DIR/candidate-distribution.json"
install_and_receipt "$candidate" candidate
publish rollback "$BASELINE_REVISION_ID" 2 clean-rollback '{"acceptance":"rollback to retained baseline"}' "$rollback"
distribute 3 "$EVIDENCE_DIR/rollback-distribution.json"
install_and_receipt "$rollback" rollback

api GET "/managed/resources/$RESOURCE_ID/publications" > "$EVIDENCE_DIR/publication-history.json"
api GET "/managed/installations?resource_id=$RESOURCE_ID" > "$EVIDENCE_DIR/receipts.json"
jq -n --arg resource "$RESOURCE_ID" --arg evidence "$EVIDENCE_DIR" '{status:"complete",resource_id:$resource,evidence_directory:$evidence,generations:[1,2,3]}'
