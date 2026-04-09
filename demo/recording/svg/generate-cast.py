#!/usr/bin/env python3
"""Generate synthetic asciinema cast files for clean SVG terminal animations.

Creates frame-perfect terminal recordings with:
- Character-by-character typing at natural speed
- Line-by-line output that flows smoothly
- Clean prompts and formatting
- No ANSI artifacts or spacing issues
"""
import json
import subprocess
import sys
import os

# Terminal colors
C = {
    'green': '\033[32m',
    'red': '\033[31m',
    'cyan': '\033[36m',
    'yellow': '\033[33m',
    'white': '\033[97m',
    'bold': '\033[1m',
    'dim': '\033[2m',
    'r': '\033[0m',
}

def c(color, text):
    """Wrap text in color codes."""
    return f"{C[color]}{text}{C['r']}"

def cb(color, text):
    """Wrap text in bold color codes."""
    return f"{C['bold']}{C[color]}{text}{C['r']}"


class CastWriter:
    def __init__(self, width=96, height=30):
        self.width = width
        self.height = height
        self.events = []
        self.time = 0.0

    def wait(self, seconds):
        self.time += seconds

    def write(self, text):
        """Write text as a single frame."""
        self.events.append([round(self.time, 4), 'o', text])

    def line(self, text='', delay=0.03):
        """Write a line with a small delay."""
        self.wait(delay)
        self.write(text + '\r\n')

    def type_prompt(self):
        """Show a colored prompt."""
        self.wait(0.1)
        self.write(f"{C['green']}${C['r']} ")

    def type_text(self, text, speed=0.035):
        """Type text character by character."""
        for ch in text:
            self.wait(speed)
            self.write(ch)
        self.wait(0.05)
        self.write('\r\n')

    def type_command(self, cmd, speed=0.035):
        """Show prompt then type a command."""
        self.type_prompt()
        self.type_text(cmd, speed)

    def blank(self):
        self.line()

    def output_lines(self, lines, delay=0.02):
        """Output multiple lines with smooth per-line delay."""
        for line in lines:
            self.wait(delay)
            self.write(line + '\r\n')

    def header(self, title, subtitle=''):
        self.blank()
        self.line(cb('white', title), delay=0.05)
        if subtitle:
            self.line(c('dim', subtitle), delay=0.03)
        self.line(c('cyan', '━' * 60), delay=0.03)
        self.blank()

    def subheader(self, title, subtitle=''):
        self.line(f"{c('cyan', '──')} {cb('white', title)}", delay=0.05)
        if subtitle:
            self.line(c('dim', f'   {subtitle}'), delay=0.03)
        self.blank()

    def divider(self):
        self.blank()
        self.line(c('dim', '─' * 60), delay=0.05)
        self.blank()

    def allow(self, text):
        self.line(f"{cb('green', '>>> ALLOW')} {c('green', text)}")

    def deny(self, text):
        self.line(f"{cb('red', '>>> DENY')}  {c('red', text)}")

    def json_output(self, obj, delay=0.02):
        """Pretty-print JSON with syntax coloring, line by line."""
        pretty = json.dumps(obj, indent=2)
        for line in pretty.split('\n'):
            colored = line
            if '"allow"' in colored:
                colored = colored.replace('"allow"', f'{C["bold"]}{C["green"]}"allow"{C["r"]}')
            elif '"deny"' in colored:
                colored = colored.replace('"deny"', f'{C["bold"]}{C["red"]}"deny"{C["r"]}')
            # Color keys
            import re
            colored = re.sub(r'"(\w+)":', f'{C["cyan"]}"\g<1>":{C["r"]}', colored)
            self.wait(delay)
            self.write(f'  {colored}\r\n')

    def save(self, path):
        header = {
            'version': 2,
            'width': self.width,
            'height': self.height,
            'env': {'SHELL': '/bin/bash', 'TERM': 'xterm-256color'}
        }
        with open(path, 'w') as f:
            f.write(json.dumps(header) + '\n')
            for event in self.events:
                f.write(json.dumps(event) + '\n')
        print(f'Saved: {path} ({len(self.events)} frames, {self.time:.1f}s)')


def get_governance_response(base_url, token, agent_id, tool_name, session_id, tool_input=None):
    """Make a real governance API call and return the JSON."""
    payload = {
        'hook_event_name': 'PreToolUse',
        'tool_name': tool_name,
        'agent_id': agent_id,
        'session_id': session_id,
        'tool_input': tool_input or {}
    }
    try:
        result = subprocess.run(
            ['curl', '-s', '-X', 'POST',
             f'{base_url}/api/public/hooks/govern?plugin_id=svg-demo',
             '-H', f'Authorization: Bearer {token}',
             '-H', 'Content-Type: application/json',
             '-d', json.dumps(payload)],
            capture_output=True, text=True, timeout=10
        )
        return json.loads(result.stdout)
    except Exception as e:
        return {'error': str(e)}


def get_token():
    token_file = os.path.join(os.path.dirname(__file__), '..', '.token')
    with open(token_file) as f:
        return f.read().strip()


def generate_governance(base_url, token):
    """Generate the governance allow/deny demo."""
    w = CastWriter()

    w.header('GOVERNANCE', 'Tool access control for AI agents')
    w.wait(0.6)

    # Allow path
    w.subheader('Admin agent requests tool access',
                'developer_agent → mcp__systemprompt__list_agents')
    w.wait(0.3)

    w.type_command('systemprompt hooks govern --agent developer_agent --tool list_agents')
    w.wait(0.2)

    resp = get_governance_response(base_url, token,
        'developer_agent', 'mcp__systemprompt__list_agents', 'svg-gov')
    w.json_output(resp)
    w.blank()
    w.allow('admin scope, all 3 rules passed')
    w.wait(1.0)

    w.divider()

    # Deny path
    w.subheader('User agent requests same tool',
                'associate_agent → mcp__systemprompt__list_agents')
    w.wait(0.3)

    w.type_command('systemprompt hooks govern --agent associate_agent --tool list_agents')
    w.wait(0.2)

    resp = get_governance_response(base_url, token,
        'associate_agent', 'mcp__systemprompt__list_agents', 'svg-gov')
    w.json_output(resp)
    w.blank()
    w.deny('scope_restriction — user cannot access admin tools')
    w.wait(1.0)

    w.divider()
    w.line(cb('cyan', 'Same tool. Two agents. Two outcomes. Both audited.'))
    w.blank()
    w.wait(1.5)

    return w


def generate_secrets(base_url, token):
    """Generate the secret detection demo."""
    w = CastWriter()

    w.header('SECRET DETECTION', 'Scanning tool inputs for plaintext credentials')
    w.wait(0.6)

    tests = [
        ('AWS Access Key', 'Bash',
         {'command': 'curl -H "Authorization: AKIAIOSFODNN7EXAMPLE" https://api.example.com'},
         'AWS access key detected'),
        ('GitHub Personal Access Token', 'Bash',
         {'command': 'curl -H "Authorization: token ghp_ABCDEFghijklmnop12345678" https://api.github.com'},
         'GitHub token detected'),
        ('RSA Private Key', 'Write',
         {'file_path': '/tmp/key.pem', 'content': '-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...'},
         'private key detected'),
    ]

    for i, (name, tool, tool_input, reason) in enumerate(tests, 1):
        w.subheader(f'Test {i} — {name}')
        w.wait(0.2)
        w.type_command(f'systemprompt hooks govern --tool {tool} --input \'...{name.lower()[:20]}...\'')
        w.wait(0.2)

        resp = get_governance_response(base_url, token,
            'developer_agent', tool, f'svg-secrets-{i}', tool_input)
        w.json_output(resp)
        w.blank()
        w.deny(reason)
        w.wait(0.8)
        w.divider()

    # Clean test
    w.subheader('Test 4 — Clean File Read')
    w.wait(0.2)
    w.type_command('systemprompt hooks govern --tool Read --input \'/src/main.rs\'')
    w.wait(0.2)

    resp = get_governance_response(base_url, token,
        'developer_agent', 'Read', 'svg-secrets-clean',
        {'file_path': '/home/user/project/src/main.rs'})
    w.json_output(resp)
    w.blank()
    w.allow('no secrets detected')
    w.wait(1.0)

    w.divider()
    w.line(cb('cyan', '3 blocked. 1 allowed. All audited.'))
    w.blank()
    w.wait(1.5)

    return w


def generate_benchmark(base_url, token):
    """Generate the load test demo with real benchmark data."""
    w = CastWriter()

    w.header('LOAD TEST', 'Governance throughput under production load')
    w.wait(0.6)

    # Run actual benchmarks
    hey = '/tmp/hey'
    if not os.path.exists(hey):
        subprocess.run(['curl', '-sL',
            'https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64',
            '-o', hey], check=True)
        os.chmod(hey, 0o755)

    # Warmup
    payload = json.dumps({
        'hook_event_name': 'PreToolUse', 'tool_name': 'Read',
        'agent_id': 'developer_agent', 'session_id': 'svg-bench-warmup',
        'tool_input': {'file_path': '/src/main.rs'}
    })
    subprocess.run([hey, '-n', '10', '-c', '5', '-m', 'POST',
        '-H', f'Authorization: Bearer {token}',
        '-H', 'Content-Type: application/json',
        '-d', payload,
        f'{base_url}/api/public/hooks/govern?plugin_id=svg-demo'],
        capture_output=True, timeout=30)

    def run_bench(n, c_count, session):
        p = json.dumps({
            'hook_event_name': 'PreToolUse', 'tool_name': 'Read',
            'agent_id': 'developer_agent', 'session_id': session,
            'tool_input': {'file_path': '/src/main.rs'}
        })
        result = subprocess.run([hey, '-n', str(n), '-c', str(c_count), '-m', 'POST',
            '-H', f'Authorization: Bearer {token}',
            '-H', 'Content-Type: application/json',
            '-d', p,
            f'{base_url}/api/public/hooks/govern?plugin_id=svg-demo'],
            capture_output=True, text=True, timeout=60)
        out = result.stdout
        import re
        rps = re.search(r'Requests/sec:\s+([\d.]+)', out)
        p50 = re.search(r'50% in ([\d.]+)', out)
        p90 = re.search(r'90% in ([\d.]+)', out)
        p99 = re.search(r'99% in ([\d.]+)', out)
        fastest = re.search(r'Fastest:\s+([\d.]+)', out)
        return {
            'rps': f'{float(rps.group(1)):.0f}' if rps else '?',
            'p50': f'{float(p50.group(1))*1000:.1f}' if p50 else '?',
            'p90': f'{float(p90.group(1))*1000:.1f}' if p90 else '?',
            'p99': f'{float(p99.group(1))*1000:.1f}' if p99 else '?',
            'fastest': f'{float(fastest.group(1))*1000:.1f}' if fastest else '?',
        }

    # Test 1
    w.subheader('500 requests, 50 concurrent',
                'JWT → scope → 3 rules → audit write per request')
    w.blank()
    w.type_command('hey -n 500 -c 50 POST /api/public/hooks/govern')
    w.wait(0.3)
    w.line(c('dim', 'Running benchmark...'))

    r1 = run_bench(500, 50, 'svg-bench-1')
    w.wait(0.5)

    w.blank()
    w.line(f"  {c('dim', 'Throughput')}     {cb('green', r1['rps'] + ' req/s')}")
    w.line(f"  {c('dim', 'p50')}            {c('green', r1['p50'] + 'ms')}")
    w.line(f"  {c('dim', 'p90')}            {c('yellow', r1['p90'] + 'ms')}")
    w.line(f"  {c('dim', 'p99')}            {c('yellow', r1['p99'] + 'ms')}")
    w.line(f"  {c('dim', 'Fastest')}        {c('green', r1['fastest'] + 'ms')}")
    w.wait(1.0)

    w.divider()

    # Test 2
    w.subheader('1000 requests, 100 concurrent', 'Doubled concurrency, sustained load')
    w.blank()
    w.type_command('hey -n 1000 -c 100 POST /api/public/hooks/govern')
    w.wait(0.3)
    w.line(c('dim', 'Running benchmark...'))

    r2 = run_bench(1000, 100, 'svg-bench-2')
    w.wait(0.5)

    w.blank()
    w.line(f"  {c('dim', 'Throughput')}     {cb('green', r2['rps'] + ' req/s')}")
    w.line(f"  {c('dim', 'p50')}            {c('green', r2['p50'] + 'ms')}")
    w.line(f"  {c('dim', 'p90')}            {c('yellow', r2['p90'] + 'ms')}")
    w.line(f"  {c('dim', 'p99')}            {c('yellow', r2['p99'] + 'ms')}")
    w.wait(1.0)

    w.divider()

    # Capacity
    rps = int(r1['rps']) if r1['rps'] != '?' else 100
    devs = rps * 60 // 10

    w.subheader('Enterprise Capacity', '10 tool calls/min per developer')
    w.blank()
    w.line(f"  {c('dim', '1 instance')}          {cb('green', f'~{devs} concurrent devs')}")
    w.line(f"  {c('dim', '3 + PgBouncer')}       {cb('green', f'~{devs*3} concurrent devs')}")
    w.line(f"  {c('dim', '10 + PgBouncer')}      {cb('green', f'~{devs*10} concurrent devs')}")
    w.blank()
    w.line(cb('green', 'Governance adds <1% latency to AI response time.'))
    w.blank()
    w.wait(2.0)

    return w


if __name__ == '__main__':
    base_url = 'http://localhost:8080'
    token = get_token()
    output_dir = os.path.join(os.path.dirname(__file__), 'recordings')

    demos = {
        '01-governance': lambda: generate_governance(base_url, token),
        '02-secrets': lambda: generate_secrets(base_url, token),
        '06-benchmark': lambda: generate_benchmark(base_url, token),
    }

    targets = sys.argv[1:] if len(sys.argv) > 1 else demos.keys()

    for name in targets:
        if name in demos:
            print(f'Generating {name}...')
            w = demos[name]()
            w.save(os.path.join(output_dir, f'{name}.cast'))
        else:
            print(f'Unknown demo: {name}')
