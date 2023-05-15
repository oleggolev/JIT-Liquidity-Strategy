import requests
import pandas as pd
import json
import time
import streamlit as st
import altair as alt
import datetime

total_swaps = 0

def fetch_data():
    response = requests.get("http://localhost:8000/get_data")
    data = json.loads(response.text)
    return data

def process_data(df):
    # Convert wei to gwei
    df['to_token_qty'] = df['to_token_qty'].astype(float) * 1e-18
    df['balance2'] = df['balance2'].astype(float) * 1e-18
    df['approve_fee'] = df['approve_fee'].astype(float) * 1e-18
    df['liq_fee'] = df['liq_fee'].astype(float) * 1e-18
    df['tx_receipt_ts'] = pd.to_datetime(df['tx_receipt_ts'], unit='ms')
    df['tx_prosessed_ts'] = pd.to_datetime(df['tx_prosessed_ts'], unit='ms')

    num_columns = ['balance1', 'balance2', 'from_token_qty', 'to_token_qty']
    for col in num_columns:
        df[col] = pd.to_numeric(df[col], errors='coerce')

    # Drop rows containing NaN values after conversion
    df = df.dropna(subset=num_columns)

    df = df[df['to_token_symbol'] == 'WETH']
    global total_swaps
    total_swaps = len(df)

    return df.query('abs((balance1 * balance2) - ((balance1 + from_token_qty) * (balance2 - to_token_qty))) <= 0.005 * (balance1 * balance2)').reset_index(drop=True)



def swap_size(df):
    df['swap_number'] = df.index + 1

    # Chart: Swap Size Over Time Line Chart
    swap_size_chart = alt.Chart(df).mark_line().encode(
        x=alt.X('swap_number:Q', axis=alt.Axis(title='Number of Swaps')),
        y=alt.Y('to_token_qty:Q', axis=alt.Axis(title='Swap Size (WETH)')),
        tooltip=[alt.Tooltip('from_token_symbol', title="Token 1"), alt.Tooltip('from_token_qty:Q', title="Token 1 Quantity", format='.2f'), alt.Tooltip('to_token_symbol', title='Token 2'), alt.Tooltip('to_token_qty:Q', title='Token 2 Quantity', format='.2f')]
    ).interactive().properties(
        width=600,
        height=400
    )

    return swap_size_chart

def calculate_individual_profit_and_cost(row):
    revenue = float(row['to_token_qty']) * 0.3 * 0.01
    if row['to_token_qty'] > 0.95:
        cost = 2 * (float(row['approve_fee']) + 104535652*1e-8)
    else:
        cost = 2 * (float(row['approve_fee']) + 7792377*1e-8)
    profit = revenue - cost

    return revenue, profit, cost

def calculate_profit(df):
    df['revenue'], df['profit'], df['cost'] = zip(*df.apply(calculate_individual_profit_and_cost, axis=1))
    df['cumulative_profit'] = df['profit'].cumsum()
    df['cumulative_cost'] = df['cost'].cumsum()
    return df


def display_cumulative_profit_chart(df):
    df['swap_number'] = df.index + 1

    # # Reshape the data to long format
    # long_df = df.melt(id_vars='swap_number', value_vars=['cumulative_profit', 'cumulative_cost'], var_name='layer', value_name='value')

    # # Merge the relevant columns from the original dataframe to long_df
    # long_df = long_df.merge(df[['swap_number', 'from_token_qty', 'from_token_symbol', 'to_token_qty', 'to_token_symbol']], on='swap_number', how='left')

    cumulative_profit_chart = alt.Chart(df).mark_line().encode(
        x=alt.X('swap_number:Q', axis=alt.Axis(title='Number of Swaps')),
        y=alt.Y('cumulative_profit:Q', title='Amount (WETH)'),
        tooltip=[alt.Tooltip('from_token_symbol', title="Token 1"), alt.Tooltip('from_token_qty:Q', title="Token 1 Quantity", format='.2f'), alt.Tooltip('to_token_symbol', title='Token 2'), alt.Tooltip('to_token_qty:Q', title='Token 1', format='.2f'), alt.Tooltip('profit:Q', title="Profit", format='.2f')]
    ).interactive().properties(
        width=1000,
        height=400
    )

    return cumulative_profit_chart

def display_profit_loss_chart(df):
    df['swap_number'] = df.index + 1

    profit_loss_chart = alt.Chart(df).mark_bar().encode(
        x=alt.X('swap_number:Q', axis=alt.Axis(title='Number of Swaps')),
        y=alt.Y('profit:Q', axis=alt.Axis(title='Profit or Loss (WETH)'), scale=alt.Scale(type='symlog')),
        color=alt.condition(
            alt.datum.profit > 0,
            alt.value('seagreen'),
            alt.value('indianred')
        ),
        tooltip=[alt.Tooltip('from_token_symbol', title="Token 1"), alt.Tooltip('from_token_qty:Q', title="Token 1 Quantity", format='.2f'), alt.Tooltip('to_token_symbol', title='Token 2'), alt.Tooltip('to_token_qty:Q', title='Token 1', format='.2f'), alt.Tooltip('profit:Q', title="Profit", format='.2f')]
    ).interactive().properties(
        width=1000,
        height=400
    )

    return profit_loss_chart

def calculate_latency(df):
    df['latency'] = df['tx_prosessed_ts'] - df['tx_receipt_ts']
    df['latency'] = df['latency'].apply(lambda x: round(x.total_seconds(), 2))
    return df

# Streamlit App
st.title("Live JIT Liquidity Dashboard")

# Create placeholders for the numbers and labels
swap_num_col1, swap_num_col2 = st.columns(2)
swap_num_col1.subheader("Total Swaps")
swap_num_col2.subheader("Valid Swaps")
num_total_swaps = swap_num_col1.empty()
num_swaps_placeholder = swap_num_col2.empty()

swaps_col1, swaps_col2 = st.columns(2)
swaps_col1.subheader("Viable Swaps")
swaps_col2.subheader("Mean Swap Size")
num_viable_swaps_placeholder = swaps_col1.empty()
mean_swap_vol_placeholder = swaps_col2.empty()


# Create a placeholder for the chart
st.subheader("Swap Volume (WETH)")
chart_placeholder = st.empty()
st.subheader("Cumulative Profit (WETH)")
cumulative_profit_chart_placeholder = st.empty()
st.subheader("Profit or Loss (WETH)")
profit_loss_chart_placeholder = st.empty()

# latency placeholders
st.markdown('***')
st.subheader("Latency Statistics (seconds)")
latency_col1, latency_col2, latency_col3 = st.columns(3)
latency_col1.subheader("Mean Latency")
latency_col2.subheader("Median Latency")
latency_col3.subheader("Latest Latency")
mean_latency_placeholder = latency_col1.empty()
max_latency_placeholder = latency_col2.empty()
latest_latency_placeholder = latency_col3.empty()
st.subheader("Latency over Time")
latency_chart_placeholder = st.empty()
st.markdown('***')

# Keep fetching and analyzing data periodically
while True:

    data = fetch_data()
    df = process_data(pd.DataFrame(eval(data)))

    # Calculate profits and filter viable swaps
    df = calculate_profit(df)
    viable_swaps = df[df['profit'] > 0]

    # Update number of swaps and viable swaps
    num_total_swaps.title(f"{total_swaps}")
    num_swaps_placeholder.title(f"{len(df)}")
    num_viable_swaps_placeholder.title(f"{len(viable_swaps)}")
    mean_swap_vol_placeholder.title(f"{round(df['to_token_qty'].mean(), 3)}")

    # Update charts
    cumulative_profit_chart = display_cumulative_profit_chart(df)
    cumulative_profit_chart_placeholder.altair_chart(cumulative_profit_chart, use_container_width=True)

    to_amount_chart = swap_size(df)
    chart_placeholder.altair_chart(to_amount_chart, use_container_width=True)

    profit_loss_chart = display_profit_loss_chart(df)
    profit_loss_chart_placeholder.altair_chart(profit_loss_chart, use_container_width=True)

    df = calculate_latency(df)
    mean_latency_placeholder.title(f"{round(df['latency'].mean(), 2)}")
    max_latency_placeholder.title(f"{round(df['latency'].median(), 2)}")
    latest_latency_placeholder.title(f"{round(df['latency'].iloc[-1], 2)}")



    latency_chart = alt.Chart(df).mark_line().encode(
            x=alt.X('tx_receipt_ts:T', axis=alt.Axis(title='Time')),
            y=alt.Y('latency:Q', axis=alt.Axis(title='Latency (s)')),
            tooltip=[alt.Tooltip('tx_receipt_ts:T', title="Time"), alt.Tooltip('latency:Q', title="Latency (s)")]
        ).interactive().properties(
            width=1000,
            height=400
        )

    latency_chart_placeholder.altair_chart(latency_chart, use_container_width=True)

    # Wait for 5 seconds before fetching data again
    time.sleep(5)
