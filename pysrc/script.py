import requests
import pandas as pd
import json
import time
import streamlit as st
import altair as alt

def fetch_data():
    response = requests.get("http://localhost:5000/get_data")
    data = json.loads(response.text)
    return data

def swap_size(df):
    df['swap_number'] = df.index + 1

    # Chart: Swap Size Over Time Line Chart
    swap_size_chart = alt.Chart(df).mark_line().encode(
        x=alt.X('swap_number:Q', axis=alt.Axis(title='Number of Swaps')),
        y=alt.Y('to_token_qty:Q', axis=alt.Axis(title='Swap Size (WETH)')),
        tooltip=['swap_number', 'to_token_qty']
    ).interactive().properties(
        width=600,
        height=400
    )

    return swap_size_chart

def calculate_individual_profit_and_cost(row):
    revenue = row['to_token_qty'] * 0.3 * 0.01
    cost = 2 * (row['approve_fee'] + row['liq_fee'])
    profit = revenue - cost

    if profit > 0:
        return profit, cost
    else:
        return 0, 0

def calculate_profit(df):
    df['profit'], df['cost'] = zip(*df.apply(calculate_individual_profit_and_cost, axis=1))
    df['cumulative_profit'] = df['profit'].cumsum()
    df['cumulative_cost'] = df['cost'].cumsum()
    return df


def display_cumulative_profit_chart(df):
    df['swap_number'] = df.index + 1

    # Reshape the data to long format
    long_df = df.melt(id_vars='swap_number', value_vars=['cumulative_profit', 'cumulative_cost'], var_name='layer', value_name='value')

    cumulative_profit_chart = alt.Chart(long_df).mark_line().encode(
        x=alt.X('swap_number:Q', axis=alt.Axis(title='Number of Swaps')),
        y=alt.Y('value:Q', title='Cumulative Value (WETH)'),
        color=alt.Color('layer:N', legend=alt.Legend(title='Layer'))
    ).interactive().properties(
        width=1000,
        height=400
    )

    return cumulative_profit_chart

# Streamlit App
st.title("Live JIT Liquidity Dashboard")

# Create placeholders for the numbers and labels
swaps_col1, swaps_col2 = st.columns(2)
swaps_col1.subheader("Number of Swaps")
swaps_col2.subheader("Viable Swaps")
num_swaps_placeholder = swaps_col1.empty()
num_viable_swaps_placeholder = swaps_col2.empty()

# Create a placeholder for the chart
st.subheader("Cumulative Swap Volume (WETH)")
chart_placeholder = st.empty()
st.subheader("Cumulative Profit (WETH)")
cumulative_profit_chart_placeholder = st.empty()

# Keep fetching and analyzing data periodically
while True:
    data = fetch_data()
    df = pd.DataFrame(data)

    # Calculate profits and filter viable swaps
    df = calculate_profit(df)
    viable_swaps = df[df['profit'] > 0]

    # Update number of swaps and viable swaps
    num_swaps_placeholder.title(f"{len(df)}")
    num_viable_swaps_placeholder.title(f"{len(viable_swaps)}")

    # Update charts
    to_amount_chart = swap_size(df)
    chart_placeholder.altair_chart(to_amount_chart, use_container_width=True)

    cumulative_profit_chart = display_cumulative_profit_chart(df)
    cumulative_profit_chart_placeholder.altair_chart(cumulative_profit_chart, use_container_width=True)

    # Wait for 5 seconds before fetching data again
    time.sleep(5)
