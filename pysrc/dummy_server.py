import random
import string
from flask import Flask, jsonify

app = Flask(__name__)

def generate_data_point():
    idx = ''.join(random.choices(string.ascii_letters + string.digits, k=5))
    exchange_type = "Exchanging"
    from_amount = random.randint(1, 100000000)
    from_token = ''.join(random.choices(string.ascii_uppercase, k=5))
    to_amount = random.randint(10000, 100000000)
    to_token = 'WETH'
    from_reserve = random.randint(1000000000, 2000000000)
    to_reserve = random.randint(100000000, 1000000000)

    data_point = {
        "id": idx,
        "exchange_type": exchange_type,
        "from_amount": from_amount,
        "from_token": from_token,
        "to_amount": to_amount,
        "to_token": to_token,
        "from_reserve": from_reserve,
        "to_reserve": to_reserve
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
