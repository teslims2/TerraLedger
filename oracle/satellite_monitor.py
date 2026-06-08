from flask import Flask, request, jsonify
import hmac
import hashlib

app = Flask(__name__)

@app.route('/webhook/earth-engine', methods=['POST'])
def earth_engine_webhook():
    signature = request.headers.get('X-GEE-Signature')
    if not signature:
        return jsonify({"error": "Missing signature"}), 401
    return jsonify({"status": "submitted on-chain"}), 200

@app.route('/health', methods=['GET'])
def health():
    return jsonify({"status": "ok"}), 200

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5001)
