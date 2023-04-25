import random
import string
import time
from flask import Flask, jsonify

app = Flask(__name__)

def generate_data_point():
    tx_hash = ''.join(random.choices(string.ascii_letters + string.digits, k=64))
    from_token_qty = random.uniform(1, 100000)
    from_token_symbol = ''.join(random.choices(string.ascii_uppercase, k=5))
    to_token_qty = random.uniform(10000, 100000)
    to_token_symbol = 'WETH'
    balance1 = random.randint(1000000000, 2000000000)
    balance2 = random.randint(100000000, 1000000000)
    approve_fee = random.uniform(0.5, 10)
    liq_fee = random.uniform(0.5, 10)
    timestamp = int(time.time())

    data_point = {
        "tx_hash": tx_hash,
        "from_token_qty": from_token_qty,
        "from_token_symbol": from_token_symbol,
        "to_token_qty": to_token_qty,
        "to_token_symbol": to_token_symbol,
        "balance1": balance1,
        "balance2": balance2,
        "approve_fee": approve_fee,
        "liq_fee": liq_fee,
        "timestamp": timestamp
    }
    return data_point


# Create an array of 1000 data points
data = []

@app.route('/get_data', methods=['GET'])
def get_data():

    for _ in range(random.randint(1,3)):
        data.append(generate_data_point())

    return jsonify(data)

if __name__ == '__main__':
    app.run(debug=False)
