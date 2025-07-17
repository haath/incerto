//! Monte Carlo simulation of the performance of various professional traders in a completely random stock market.
//!
//! This example showcases interactions over different entity types (stocks and traders), as well as the sampling
//! and plotting of time series data.
//!
//! Specifically in this simulation:
//!
//! * A number of traders are spawned, starting with the same amount of cash and empty portfolios.
//! * A number of stocks are spawned, starting at the same price.
//! * Each simulation step represents one day, during which:
//!     * Every stock's price changes according to a normal distribution.
//!         * Stocks that reach a price of `0.0` become delisted and don't update anymore.
//!     * Every trader looks at every available stock, and decides whether to buy it with some probability.
//!         * The amount of shares to buy is sampled uniformly from a range.
//!     * Every trader looks at every stock in his portfolio, and decides whether to sell all his shares with some probability.
//!
//! After running the simulation, the time series of the net worth for all traders is collected and plotted.
//!
//! Note that the plotting is done using the [plotters](https://github.com/plotters-rs/plotters) crate, which is
//! not a dependency of incerto.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::cast_precision_loss)]
use std::{collections::HashMap, ops::RangeInclusive};

use incerto::prelude::*;
use plotters::{
    prelude::*,
    style::full_palette::{AMBER, INDIGO, ORANGE, PURPLE},
};
use rand::prelude::*;
use rand_distr::Normal;

const SIMULATION_STEPS: usize = 2 * 365;
const NUM_TRADERS: usize = 10;
const NUM_STOCKS: usize = 100;

const TRADER_INITIAL_CASH: f64 = 100.0;
const TRADER_CHANCE_BUY_STOCK: f64 = 0.01;
const TRADER_CHANCE_SELL_STOCK: f64 = 0.005;
const TRADER_BUY_NUM_SHARES: RangeInclusive<usize> = 1..=5;

const STOCK_INITIAL_PRICE: f64 = 5.0;
const STOCK_PRICE_CHANGE_MEAN: f64 = 0.001; // small positive bias due to inflation
const STOCK_PRICE_CHANGE_STD_DEV: f64 = 0.2;

const PLOT_STORE_PATH: &str = "images/trader_net_worths.png";

#[derive(Debug, Component, Hash, PartialEq, Eq, Clone, Copy)]
struct StockId(usize);

#[derive(Debug, Component)]
struct StockPrice(f64);

#[derive(Debug, Component)]
struct StockDelisted;

#[derive(Debug, Component, Hash, PartialEq, Eq, Clone, Copy)]
struct TraderId(usize);

#[derive(Debug, Component)]
struct Trader
{
    cash: f64,
    portfolio: HashMap<StockId, usize>, // #shares owned of each stock
}

/// A trader's actual networth requires knowing their portfolio, as well as the prices for all
/// the stocks that they own.
/// So to make it easier to [`Sample`] the net worth we create this auxiliary component which
/// will carry each trader's net worth computed each step using a system.
#[derive(Debug, Component)]
struct TraderNetWorth(TraderId, f64);

impl SampleAggregate<HashMap<TraderId, f64>> for TraderNetWorth
{
    /// Collect the net worth values from all traders into a hash map,
    /// mapping each trader id to his corresponding net worth.
    fn sample_aggregate(components: &[&Self]) -> HashMap<TraderId, f64>
    {
        components.iter().map(|c| (c.0, c.1)).collect()
    }
}

fn main() -> Result<(), SimulationError>
{
    let mut simulation = SimulationBuilder::new()
        // traders
        .add_entity_spawner(spawn_traders)
        .add_systems((
            traders_may_buy_shares,
            traders_may_sell_shares,
            traders_calculate_net_worth,
        ))
        // stocks
        .add_entity_spawner(spawn_stocks)
        .add_systems(stocks_price_change)
        .record_aggregate_time_series::<TraderNetWorth, _>(1)?
        .build();

    simulation.run(SIMULATION_STEPS);

    let time_series = simulation.get_aggregate_time_series::<TraderNetWorth, _>()?;
    plot_net_worths(&time_series);

    Ok(())
}

fn spawn_traders(spawner: &mut Spawner)
{
    for id in 0..NUM_TRADERS
    {
        spawner.spawn((
            TraderId(id),
            Trader {
                cash: TRADER_INITIAL_CASH,
                portfolio: HashMap::new(),
            },
            TraderNetWorth(TraderId(id), TRADER_INITIAL_CASH),
        ));
    }
}

fn spawn_stocks(spawner: &mut Spawner)
{
    for id in 0..NUM_STOCKS
    {
        spawner.spawn((StockId(id), StockPrice(STOCK_INITIAL_PRICE)));
    }
}

/// During each step, the prices of all stocks move by a random amount.
fn stocks_price_change(
    mut commands: Commands,
    mut query: Query<(Entity, &mut StockPrice), Without<StockDelisted>>,
)
{
    let mut rng = rand::rng();

    // sample the deltas from a normal distribution
    let price_change_distribution =
        Normal::new(STOCK_PRICE_CHANGE_MEAN, STOCK_PRICE_CHANGE_STD_DEV).unwrap();

    for (stock, mut stock_price) in &mut query
    {
        let price_change = rng.sample(price_change_distribution);

        stock_price.0 = (stock_price.0 + price_change).max(0.0);

        if stock_price.0 < f64::EPSILON
        {
            // a stock is delisted permanently if its price hits zero
            commands.entity(stock).insert(StockDelisted);
        }
    }
}

fn traders_may_buy_shares(
    mut query: Query<&mut Trader>,
    query_stocks: Query<(&StockId, &StockPrice), Without<StockDelisted>>,
)
{
    let mut rng = rand::rng();

    for mut trader in &mut query
    {
        for (stock_id, stock_price) in &query_stocks
        {
            let price = stock_price.0;
            let already_owned = trader.portfolio.contains_key(stock_id);
            let decides_to_buy = rng.random_bool(TRADER_CHANCE_BUY_STOCK);

            if !already_owned && decides_to_buy
            {
                let num_shares = rng.random_range(TRADER_BUY_NUM_SHARES);
                let amount = (num_shares as f64) * price;
                if amount > trader.cash
                {
                    // we skip the cases where the trader decides to buy an amount greater than his available cash
                    continue;
                }

                // buy the stock
                trader.portfolio.insert(*stock_id, num_shares);
                trader.cash -= amount;
                assert!(trader.cash >= 0.0);
            }
        }
    }
}

fn traders_may_sell_shares(
    mut query: Query<&mut Trader>,
    query_stocks: Query<(&StockId, &StockPrice)>,
)
{
    let mut rng = rand::rng();

    // build a stock-price lookup table
    let stock_prices: HashMap<StockId, f64> = query_stocks
        .iter()
        .map(|(id, price)| (*id, price.0))
        .collect();

    for mut trader in &mut query
    {
        // go through all owned stocks and decide randomly which ones to sell
        let mut to_sell = Vec::new();
        for (stock_id, num_shares) in &trader.portfolio
        {
            let decides_to_sell = rng.random_bool(TRADER_CHANCE_SELL_STOCK);

            if decides_to_sell
            {
                to_sell.push((*stock_id, *num_shares));
            }
        }

        // sell them by removing them from the portfolio and incrementing the cash
        // by the sale amount
        for (stock_id, num_shares) in to_sell
        {
            trader.portfolio.remove(&stock_id);

            let price = stock_prices[&stock_id];
            trader.cash += (num_shares as f64) * price;
        }
    }
}

/// Computes each trader's net worth and stores in the auxiliary [`TraderNetWorth`] component.
fn traders_calculate_net_worth(
    mut query: Query<(&Trader, &mut TraderNetWorth)>,
    query_stocks: Query<(&StockId, &StockPrice)>,
)
{
    // build a stock-price lookup table
    let stock_prices: HashMap<StockId, f64> = query_stocks
        .iter()
        .map(|(id, price)| (*id, price.0))
        .collect();

    for (trader, mut net_worth) in &mut query
    {
        let portfolio_value = trader
            .portfolio
            .iter()
            .map(|(stock_id, &num_shares)| {
                let stock_price = stock_prices[stock_id];
                (num_shares as f64) * stock_price
            })
            .sum::<f64>();

        net_worth.1 = trader.cash + portfolio_value;
    }
}

fn plot_net_worths(time_series: &[&HashMap<TraderId, f64>])
{
    // convert the data to one time series per trader
    let mut time_series_per_trader: HashMap<TraderId, Vec<f64>> = time_series
        .first()
        .expect("no time series recorded")
        .keys()
        .map(|&id| (id, Vec::<f64>::new()))
        .collect();
    for point in time_series
    {
        for (id, &value) in *point
        {
            let series = time_series_per_trader
                .get_mut(id)
                .expect("missing trader time series");
            series.push(value);
        }
    }

    // expect them all to have equal length
    let series_length = time_series_per_trader[&TraderId(0)].len();
    for series in time_series_per_trader.values()
    {
        assert_eq!(series.len(), series_length);
    }

    let root_area = BitMapBackend::new(PLOT_STORE_PATH, (1024, 512)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let mut ctx = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption(
            "Traders' net worth over a 2-year period",
            ("sans-serif", 40),
        )
        .build_cartesian_2d(0..series_length, 0.0..(2.0 * TRADER_INITIAL_CASH))
        .unwrap();

    ctx.configure_mesh().draw().unwrap();

    let colors = [&RED, &GREEN, &BLUE, &PURPLE, &ORANGE, &INDIGO, &AMBER];

    for (id, series) in time_series_per_trader
    {
        let color = colors[id.0 % colors.len()];
        let series_iter = series.into_iter().enumerate();

        ctx.draw_series(LineSeries::new(series_iter, *color))
            .unwrap();
    }

    root_area.present().unwrap();
}
