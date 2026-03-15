from flask import Flask, request, Response
import requests

app = Flask(__name__)

KIMI_BASE = "https://api.kimi.com/coding/v1"

@app.route('/<path:path>', methods=['GET', 'POST', 'PUT', 'DELETE'])
def proxy(path):
    url = f"{KIMI_BASE}/{path}"
    headers = dict(request.headers)
    headers['User-Agent'] = 'claude-code/0.1.0'  # Spoof the agent
    headers.pop('Host', None)
    
    resp = requests.request(
        method=request.method,
        url=url,
        headers=headers,
        data=request.get_data(),
        params=request.args,
        stream=True
    )
    
    return Response(
        resp.iter_content(chunk_size=8192),
        status=resp.status_code,
        headers=dict(resp.headers)
    )

if __name__ == '__main__':
    app.run(port=8787)

