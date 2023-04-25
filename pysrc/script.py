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

def display_swap_size_chart(df):
    df['swap_number'] = df.index + 1

    # Chart: Swap Size Over Time Line Chart
    swap_size_chart = alt.Chart(df).mark_line().encode(
        x=alt.X('swap_number:Q', axis=alt.Axis(title='Number of Swaps')),
        y=alt.Y('to_amount:Q', axis=alt.Axis(title='Swap Size (WETH)')),
        tooltip=['swap_number', 'to_amount']
    ).interactive().properties(
        title='Swap Size Over Time (WETH)',
        width=600,
        height=400
    )

    return swap_size_chart


# Streamlit App
st.title("Live JIT Liquidity Dashboard")


# Create a placeholder for the chart
chart_placeholder = st.empty()

# Keep fetching and analyzing data periodically
while True:
    data = fetch_data()
    df = pd.DataFrame(data)
    to_amount_chart = display_swap_size_chart(df)

    # Update chart
    chart_placeholder.altair_chart(to_amount_chart, use_container_width=True)

    # Wait for 5 seconds before fetching data again
    time.sleep(5)
