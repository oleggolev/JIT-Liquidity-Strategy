# JIT Dashboard
## Run

Start the rust server and install and run the streamlit app:

```
# create virtual environment
python3 -m venv venv

pip install streamlit

streamlit run script.py
```

If you want to run the dummy server to get fake data, run this in another terminal:
```
pip install Flask

python dummy_server.py
```

## Info
### Numbers
    1. Number of incoming swap transactions.
    2. Number of swaps viable for JIT Liquidity

### Charts
    1. Swap volume
    2. Cumulative profit
        2a. Cumulative cost of JIT attack

