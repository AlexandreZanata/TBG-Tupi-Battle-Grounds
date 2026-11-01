"""Version the complete static JS/CSS graph so cached code cannot outlive new page markup."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import posixpath
import re
from urllib.parse import parse_qsl, urlencode, urlsplit, urlunsplit

TEXT = {'.html', '.js', '.mjs', '.css', '.json'}
CODE = {'.js', '.mjs', '.css'}
REFERENCES = re.compile(r'''(?P<quote>["'`])(?P<url>(?:\.{1,2}/|/)[^"'`\s]+)(?P=quote)''')


def rendered(mapping):
    payload = {name: source.read_bytes() for name, source in mapping.items()}
    inputs = [(name, hashlib.sha256(body).hexdigest()) for name, body in sorted(payload.items())
              if PurePosixPath(name).suffix in TEXT]
    inputs.append(('renderer', hashlib.sha256(Path(__file__).read_bytes()).hexdigest()))
    revision = hashlib.sha256(json.dumps(inputs, separators=(',', ':')).encode()).hexdigest()

    def rewrite(name, match):
        ref = urlsplit(match['url'])
        target = posixpath.normpath(posixpath.join(posixpath.dirname('/' + name), ref.path)).lstrip('/')
        if ref.scheme or ref.netloc or target not in payload or PurePosixPath(target).suffix not in CODE:
            return match[0]
        query = [(key, value) for key, value in parse_qsl(ref.query) if key != 'v']
        query.append(('v', revision))
        versioned = urlunsplit(('', '', ref.path, urlencode(query), ref.fragment))
        return match['quote'] + versioned + match['quote']

    for name, body in payload.items():
        if PurePosixPath(name).suffix in {'.html', '.js', '.mjs', '.css'}:
            payload[name] = REFERENCES.sub(lambda match: rewrite(name, match), body.decode('utf-8')).encode('utf-8')
    return payload
