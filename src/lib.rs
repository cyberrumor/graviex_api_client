use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::error::Error;
use std::str;
use std::hash::BuildHasher;
use itertools::Itertools;
use sha2::Sha256;
// use hmac::{Hmac, Mac, NewMac};
use hmac::{Hmac, Mac};

extern crate serde;
use serde::{Serialize, Deserialize};

include!("creds.rs");
include!("test.rs");


static mut SEED: usize = 0;

/// These structs are response formats from graviex's api.
/// We use them so we can more easily deserialize responses with serde.
#[derive(Serialize, Deserialize, Debug)]
pub struct MarketList {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Depth {
    pub timestamp: usize, // unix timestamp of data grab
    pub asks: Vec<TinyOrder>, // [['1000.0', '0.001'], ['419.0', 0.1511']]
    pub bids: Vec<TinyOrder>, // same as above
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TinyOrder {
    pub price: String,
    pub vol: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Market {
    pub attributes: Attributes,
    // pub sort_order: usize,
    // pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attributes {
    pub id: String,
    pub code: usize,
    pub name: String,
    pub base_unit: String,
    pub quote_unit: String,
    pub bid: Bid,
    pub ask: Ask,
    pub sort_order: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bid {
    pub fee: f64,
    pub currency: String,
    pub fixed: usize,
    pub lot: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ask {
    pub fee: f64,
    pub currency: String,
    pub fixed: usize,
    pub lot: f64,
}



#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Ticker {
    pub name: String, // like "GIO/BTC"
    pub base_unit: String, // like "gio"
    pub base_fixed: usize, // number of decimal places considered
    pub base_fee: f64, // fee, typically 0.002, percentual
    pub quote_unit: String, // like "btc"
    pub quote_fixed: usize, // number of decimal places considered
    pub quote_fee: f64, // fee, typically 0.002, percentual
    pub api: bool, // whether accessible via api or not
    pub base_lot: Option<f64>, // no idea what this is for, int or null typically
    pub quote_lot: Option<f64>, // no idea what this is for, int or null typically
    pub base_min: String, // minimum fee we must exceed to have a valid trade
    pub quote_min: String, // minimum fee we must exceed to have a valid trade
    pub blocks: usize, // perhaps having to due with number of orders waiting, or chain stats
    pub block_time: String, // time like "2021-07-12 12:49:13", might be empty ""
    pub wstatus: String, // on or off
    pub low: String, // lowest price in last 24h
    pub high: String, // highest price in last 24h
    pub last: String, // last price
    pub open: String, // currently available price
    pub volume: String, // trade volume in last 24h
    pub volume2: String, // second coin volume in 24h
    pub sell: String, // available buy price
    pub buy: String, // avaiable sell price
    pub at: usize, // unix timestamp of data like 1626125887
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Member {
    pub sn: String, // unique identifier of user
    pub name: Option<String>, // username
    pub email: String, // user email
    pub activated: bool, // whether user is activated
    pub accounts_filtered: Vec<Account>, // user's accounts info
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub currency: String, // account type like btc or usd
    pub balance: String, // excludes locked funds
    pub locked: String, // locked funds
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub id: usize, // unique order ID
    pub side: String, // buy or sell
    pub price: String, // order price,
    pub avg_price: String, // average execution price
    pub state: String, // wait, done, or cancel
    pub market: String, // which market the order belongs to
    pub created_at: String, // 2014-04-18T02:02:33Z formatted creation date
    pub volume: String, // volume to buy/sell, == remaining_volume + executed_volume
    pub remaining_volume: String, // remaining volume, always <= to volume
    pub executed_volume: String, // fulfilled volume, always <= volume
    pub trades: Option<Vec<Trade>>, // the order's trade history. only some results have
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trade {
    pub id: usize, // unique ID
    pub price: String, // trade pricec
    pub volume: String, // trade volume
    pub market: String, // like btcusd
    pub created_at: String, // time formatted like 2014-04-18T02:02:33Z
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderBook {
    pub asks: Vec<Order>,
    pub bids: Vec<Order>,
}

fn graviex_handler<S: BuildHasher>(
req_method: &str,
api_target: &str,
data: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {

    type HmacSha256 = Hmac<Sha256>;

    // populate q with any values that were passed via data arg
    // this step is necessary because sometimes data will be empty
    let mut q: HashMap<&str, &str> = HashMap::new();
    for (key, value) in data {
        q.insert(key, value);
    }

    // we're single threaded and using an unsafe block to edit a mutable static
    // which should be memory safe
    unsafe {
        // get a unique ending to the unix timestamp
        if SEED < 998 {
            SEED += 1;
        } else {
            SEED = 0;
        }
        let mut adjusted_time = SEED.to_string();
        while adjusted_time.len() < 3 {
            adjusted_time.insert(0, '0');
        }
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let mut tonce = current_time.as_secs().to_string();
        tonce.push_str(&adjusted_time);


        // populate q with our tonce and access key
        q.insert("tonce", &tonce);
        q.insert("access_key", &GRAVIEX_KEY);

        // populate query string and query dict with values from q
        let mut query_string: String = String::new();
        let mut query_dict: HashMap<&str, &str> = HashMap::new();
        for (key, value) in q.iter().sorted() {
            query_string.push_str(&key);
            query_string.push('=');
            query_string.push_str(&value);
            query_string.push('&');
            query_dict.insert(key, value);
        }

        // get rid of the trailing '&' on query_string
        let query = if query_string.is_empty() {
            // query string was empty, prevent neg index on empty string
            "".to_string()
        } else {
            query_string.split_at(query_string.len() - 1).0.to_string()
        };

        // appease the API overlords with their message syntax
        let mut message: String = String::new();
        message.push_str(req_method);
        message.push('|');
        message.push_str(api_target);
        message.push('|');
        message.push_str(&query);

        // give that bad boy some hmac signature action
        // type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(GRAVIEX_SECRET.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(&message.as_bytes());

        // let sig = mac.finalize().into_bytes();
        let signature = mac.finalize().into_bytes();
        let sig = hex::encode(signature);

        // add our signature to the end of the request
        query_dict.insert("signature", &sig);
        // println!("query_dict: {:?}", query_dict);

        let mut url: String = "https://graviex.net".to_string();
        url.push_str(api_target);


        if req_method == "GET" {
            // make a get request
            let mut request = minreq::get(&url).with_timeout(2);
            for (key, value) in &query_dict {
                request = request.with_param((*key).to_string(), (*value).to_string());
            }
            let response = request.send()?;

            return Ok(response.as_str()?.to_string());

        }

        // make a post request
        let mut request = minreq::post(&url).with_timeout(2);
        for (key, value) in &query_dict {
            request = request.with_param((*key).to_string(), (*value).to_string());
        }
        let response = request.send()?;
        return Ok(response.as_str()?.to_string());

    }
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn markets() -> Result<Vec<MarketList>, Box<dyn Error>> {
    println!("markets() was called");
    let response = minreq::get("https://graviex.net/webapi/v3/markets.json")
        .send()?
        .as_str()?
        .to_string();
    let result: Vec<MarketList> = serde_json::from_str(&response)?;
    Ok(result)

}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn market(m: &str) -> Result<Market, Box<dyn Error>> {
    println!("market({:?}) was called", &m);
    let mut url: String = "https://graviex.net/webapi/v3/markets/".to_string();
    url.push_str(m);
    url.push_str(&".json");
    let response = minreq::get(url)
        .send()?
        .as_str()?
        .to_string();
    let result: Market = serde_json::from_str(&response)?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn tickers() -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
    println!("tickers() was called");
    let response = minreq::get("https://graviex.net:443/webapi/v3/tickers.json")
        .send()?
        .as_str()?
        .to_string();
    let result: HashMap<String, Ticker> = serde_json::from_str(&response)?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn ticker(t: &str) -> Result<Ticker, Box<dyn Error>> {
    println!("ticker({:?}) was called", &t);
    let mut url: String = "https://graviex.net:443/webapi/v3/tickers/".to_string();
    url.push_str(t);
    url.push_str(&".json");
    let response = minreq::get(url)
        .send()?
        .as_str()?
        .to_string();
    let result: Ticker = serde_json::from_str(&response)?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn me() -> Result<Member, Box<dyn Error>> {
    println!("me() was called");
    let response = graviex_handler(
        "GET",
        "/webapi/v3/members/me.json",
        HashMap::new()
    )?;
    let result: Member = serde_json::from_str(&response)?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn register_device(device_id: &str) -> Result<String, minreq::Error> {
    println!("register_device({:?}) was called", &device_id);
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("device", &device_id);
    let result = graviex_handler(
        "POST",
        "/webapi/v3/members/me/register_device.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn history<S: BuildHasher>(map: HashMap<&str, &str, S>) -> Result<String, minreq::Error> {
    // optional params:
    // currency: str = any ticker name characters
    // limit: usize = number of returned records, default is 100
    // type: str = ["withdrawal", "deposit"]
    // from: str = date/time
    // to: str = date/time
    // page: usize = specify page of paginated results
    // order_by: str = orders results ["asc", "des"]
    println!("history({:?}) was called", map);
    let result = graviex_handler(
        "GET",
        "/webapi/v3/account/history.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn deposits<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    // optional params:
    // currency: list (comma separated) = gio,btc,doge,lts,dev
    // limit: usize = number of returned records, default 100
    // sate: str = unknown, but likely "settled", "unlocked", etc
    println!("deposits({:?}) was called", map);
    let result = graviex_handler(
        "GET",
        "/webapi/v3/deposits.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn deposit(txid: &str) -> Result<String, minreq::Error> {
    // gets details of a specific deposit
    println!("deposit({:?}) was called", &txid);
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("txid", &txid);
    let result = graviex_handler(
        "GET",
        "/webapi/v3/deposit.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn deposit_address(currency: &str) -> Result<String, minreq::Error> {
    // gets your graviex deposit address for every coni in currency
    println!("deposit_address({:?}) was called", &currency);
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("currency", &currency);
    let result = graviex_handler(
        "GET",
        "/webapi/v3/deposit_address.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn gen_deposit_address(currency: &str) -> Result<String, minreq::Error> {
    // result is async so you can try to call deposit_address until wallet exists
    println!("gen_deposit_address({:?}) was called", &currency);
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("currency", &currency);
    let result = graviex_handler(
        "GET",
        "/webapi/v3/gen_deposit_address.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn orders_get<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<Vec<Order>, Box<dyn Error>> { 
    // gets only your own orders
    // optional params:
    // market: str = unique market id, xxxxxx, like btcusd
    // state: str = filter order by state, default to 'wait' (active orders)
    // limit: usize = limit the number o freturned orders, default 100
    // page: usize = specify page of paginated results
    // order_by: str = if set, returned orders will be sorted ["asc", "des"]
    println!("orders_get({:?}) was called", map);
    let response = graviex_handler(
        "GET",
        "/webapi/v3/orders.json",
        map
    )?;
    let result: Vec<Order> = serde_json::from_str(&response)?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn orders_post<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    // required params:
    // market: str = unique market ID, ie "btcusd"
    // side: str = "sell" or "buy"
    // volume: str = the amount you want to buy/sell
    // // an order could be paritally executed, e.g. an order to sell 5 btc
    // // can be matched with a buy 3 btc order, left 2 btc to be sold;
    // // in this case, the order's volume would be 5.0, remaining_volume would
    // // be 2.0, executed volume would be 3.0.
    //
    // optional params:
    // price: str = price for each unit, e.g. if you want to sell/buy one btc
    // // at 3000 CNY, the price is 3000.0.
    // ord_type: str = unknown
    println!("orders_post({:?}) was called", map);
    let result = graviex_handler(
        "POST",
        "/webapi/v3/orders.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn orders_history<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<Vec<Order>, Box<dyn Error>> {
    // optional params:
    // market: str = any market name chars
    // state: str = 'wait', 'done', 'cancel'
    // limit: usize = number of returned results, default 100
    // from: str = from date/time
    // to: str = to date/time 
    // page: usize = specify the page of paginated results
    // order_by: str = ['des', 'asc']
    println!("orders_history({:?}) was called", map);
    let response = graviex_handler(
        "GET",
        "/webapi/v3/orders/history.json",
        map
    )?;
    let result: Vec<Order> = serde_json::from_str(&response)?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn orders_multi<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    // required params:
    // orders: HashMap = {'side': 'buy'|'sell', 'volume': str}
    //
    // optional params:
    // orders: HashMap = {'price': str, 'ord_type': str (probably 'market')}
    println!("orders_multi({:?}) was called", map);
    let result = graviex_handler(
        "POST",
        "/webapi/v3/orders/multi.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn orders_clear(side: &str) -> Result<String, minreq::Error> {
    // cancel all orders of specific type. side expects 'buy' or 'sell'
    println!("orders_clear({:?}) was called", &side);
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("side", &side);
    let result =  graviex_handler(
        "POST",
        "/webapi/v3/orders/clear.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn order(order_id: &str) -> Result<String, minreq::Error> {
    // get information of specified order
    println!("order({:?}) was called", &order_id);
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("order_id", &order_id);
    let result = graviex_handler(
        "GET",
        "/webapi/v3/order.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn order_delete(order_id: &str) -> Result<String, minreq::Error> {
    // delete target order_id
    println!("order_delete({:?}) was called", &order_id);
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("order_id", &order_id);
    let result = graviex_handler(
        "POST",
        "/webapi/v3/order/delete.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn order_book<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<OrderBook, Box<dyn Error>> {
    // get the order book of the specified market
    // required keys;
    // 'market': str = unique market id like 'btcusd'
    //
    // optional keys:
    // 'asks_limit': usize = limit number of returned sell orders, default 20
    // 'bids_limit': usize = limit number of returned buy orders, default 20
    println!("order_book({:?}) was called", map);
    let response = graviex_handler(
        "GET",
        "/webapi/v3/order_book.json",
        map
    )?;
    let result: OrderBook = serde_json::from_str(&response)?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn depth<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<Depth, Box<dyn Error>> {
    // get depth of specified market. both asks and bids are sorted high to low
    // required keys:
    // 'market': str = unique market id like 'btcusd'
    //
    // optional keys:
    // 'limit': usize = limit number of returned price intervals, default 100
    // 'order': 'asc' or 'des'
    println!("depth({:?}) was called", map);

    let mut request = minreq::get("https://graviex.net:443/webapi/v3/depth.json")
        .with_timeout(2);
    for (key, value) in map {
        request = request.with_param(key.to_string(), &value.to_string());
    }
    let response = request.send()?;

    let result = serde_json::from_str(&response.as_str()?)?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn trades<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    // get recent trades on market, deduplicated, reverse creation order
    // required keys:
    // 'market': unique market id like "btcusd"
    //
    // optional keys:
    // limit: usize = default 50
    // timestamp: usize = unix epoch like graviex_handler + 000,
    // // return only trades that were executed before this time
    // from: usize = trade_id. If set, only trades created after will return
    // to: usize = trade_id. If set, only trades created before will return.
    // order_by: &str, either 'asc' or 'des'
    println!("trades({:?}) was called", &map);

    let mut request = minreq::get("https://graviex.net/webapi/v3/trades.json")
        .with_timeout(2);
    for (key, value) in map {
        request = request.with_param(key.to_string(), &value.to_string());
    }
    let response = request.send()?;
    return Ok(response.as_str()?.to_string());
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn trades_my<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    // get your executed trades history, results are paginated
    // required keys
    // market: &str = "btcusd"
    //
    // optional keys:
    // limit: usize = default 50
    // timestamp: usize = unix epoch like graviex_handler + 100
    // from: usize = trade_id. If set, only trades created after will return
    // to: usize = trade_id. If set, only trades created before will return
    // order_by: &str = either 'des' or 'asc'
    println!("trades_my({:?}) was called", &map);
    let result = graviex_handler(
        "GET",
        "/webapi/v3/trades/my.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn trades_history<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    // get recent trades from market, deduplicated, sorted in reverse creation order.
    // optional keys:
    // market, limit, from, to, page, order_by
    println!("trades_history({:?}) was called", &map);
    let result = graviex_handler(
        "GET",
        "/webapi/v3/trades/history.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn trades_simple(market: &str) -> Result<String, minreq::Error> {
    // get recent trades on market with minimal properties
    // deduplicated, reverse creation order
    println!("trades_simple({:?}) was called", &market);
    let response = minreq::get("https://graviex.net/webapi/v3/trades_simple.json")
        .with_param("market", market)
        .with_timeout(2)
        .send()?;
    return Ok(response.as_str()?.to_string());
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn kline<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    // required keys:
    // market: &str = btcusd
    // trade_id: usize = trade_id, id of the first trade you received
    //
    // optional keys:
    // limit: usize = default 20
    // period: usize = 1 (default), 5, 15, 30, 60, 120, 240, 360, 720, 1440, 4320, 10080
    // timestamp: usize = unix timestamp, return only trades created more recently than
    println!("kline({:?}) was called", &map);
    let mut request = minreq::get("https://graviex.net/webapi/v3/k.json")
        .with_timeout(2);
    for (key, value) in map {
        request = request.with_param(key.to_string(), &value.to_string());
    }
    let response = request.send()?;
    return Ok(response.as_str()?.to_string());
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn kline_pending<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    // required keys:
    // market: &str = btcusd
    // trade_id: usize = trade_id, id of the first trade you received
    //
    // optional keys:
    // limit: usize = default 20
    // period: usize = 1 (default), 5, 15, 30, 60, 120, 240, 360, 720, 1440, 4320, 10080
    // timestamp: usize = unix timestamp, return only trades created more recently than
    println!("kline_pending({:?}) was called", &map);
    let result = graviex_handler(
        "GET",
        "/webapi/v3/k_with_pending_trades.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn timestamp() -> Result<usize, Box<dyn Error>> {
    println!("timestamp() was called");
    let result = minreq::get("https://graviex.net/webapi/v3/timestamp.json")
        .send()?
        .as_str()?
        .parse::<usize>()?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn settings_get() -> Result<String, minreq::Error> {
    println!("settings_get() was called");
    let result = graviex_handler(
        "GET",
        "/webapi/v3/settings/get.json",
        HashMap::new()
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn settings_store<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    println!("settings_store({:?}) was called", &map);
    // possible keys:
    // darkmode: bool
    // was_quick_tour: bool
    // filter_favorites: bool
    // markets_filter (defaults to "all")
    // sound: bool
    // candlestick_scale: usize = defaults to 1D
    // candlestick_timezone: defaults to "exchange"
    // // and a list of favorite pairs where key name is the fav-pairname
    // fav-vrscbtc: bool
    println!("settings_store({:?}) was called", &map);
    let mut map_as_str: String = "{".to_string();
    for (key, value) in map {
        // we need the final string to have quotes around keys/values
        map_as_str.push_str(key);
        map_as_str.push(':');
        map_as_str.push_str(value);
        map_as_str.push(',');
    }
    map_as_str.push('}');
    println!("map_as_str: {:?}", &map_as_str);

    let mut newmap: HashMap<&str, &str> = HashMap::new();
    newmap.insert("data", &map_as_str);

    let result = graviex_handler(
        "POST",
        "/webapi/v3/settings/store.json",
        // in python this is
        // {'data': str(json.dumps(map)).replace(' ', '')}
        newmap
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn currency_info(coin: &str) -> Result<String, minreq::Error> {
    println!("curency_info({:?}) was called", &coin);
    let response = minreq::get("https://graviex.net/webapi/v3/currency/info.json")
        .with_timeout(2)
        .with_param("currency", coin)
        .send()?
        .as_str()?
        .to_string();

    Ok(response)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn withdraws<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    // required keys:
    // 'currency': &str = like 'btc'
    //
    // optional keys:
    // limit: usize = max number of results, default is probably 100
    // state: &str = unknown, probably 'pending' 'complete' or 'locked'
    println!("withdraws({:?}) was called", &map);
    let response = graviex_handler(
        "GET",
        "/webapi/v3/withdraws.json",
        map
    )?;
    Ok(response)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn create_withdraw<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    println!("create_withdraw({:?}) was called", &map);
    // make withdrawal.
    // required keys:
    // currency: &str = 'btc' or 'vrsc'
    // fund_uid: &str = the address to withdraw to
    // sum: &str = amount, string with format "0.0000"
    //
    // optional keys:
    // provider: withdaw providor, unknown what this means
    // speed_up: accelerate window, unknown type
    println!("create_withdraw({:?}) was called", &map);
    let result = graviex_handler(
        "POST",
        "/webapi/v3/create_withdraw.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn fund_sources(currency: &str) -> Result<String, minreq::Error> {
    println!("fund_sources({:?}) was called", currency);
    // currency is a string like 'gio' btc' or 'vrsc'
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("currency", currency);
    let result = graviex_handler(
        "GET",
        "/webapi/v3/fund_sources.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn fund_source_create<S: BuildHasher>(map: HashMap<&str, &str, S>)
-> Result<String, minreq::Error> {
    println!("fund_source_create({:?}) was called", &map);
    // required keys:
    // currency: &str = 'gio' or 'btc' or 'doge'
    // uid: &str = address of fund source
    // extra: &str = label you want to assign to it
    //
    // optional keys:
    // fund-uid: &str = provider, unknown what this means
    println!("fund-source_create({:?}) was called", &map);
    let result = graviex_handler(
        "POST",
        "/webapi/v3/create_fund_source.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn fund_source_remove(source_id: &str) -> Result<String, minreq::Error> {
    // delete a fund source
    println!("fund_source_remove({:?}) was called", source_id);
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("id", source_id);
    let result = graviex_handler(
        "POST",
        "/webapi/v3/remove_fund_source.json",
        map
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn strategies_list() -> Result<String, minreq::Error> {
    println!("strategies_list() was called");
    let result = graviex_handler(
        "GET",
        "/webapi/v3/strategies/list.json",
        HashMap::new()
    )?;
    Ok(result)
}

/// # Errors
/// returns `minreq::Error` if anything goes wrong
#[allow(unused)]
pub fn strategies_list_my() -> Result<String, minreq::Error> {
    println!("strategies_list_my() was called");
    let result = graviex_handler(
        "GET",
        "/webapi/v3/strategies/my.json",
        HashMap::new()
    )?;
    Ok(result)
}


