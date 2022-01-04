#[allow(unused)]
pub fn test_no_params() {
    // no_params
    {
        let resp = markets();
        println!("{:?}", resp);
    }
    {
        let resp = tickers();
        println!("{:?}", resp);
    }
    {
        let resp = me();
        println!("{:?}", resp);
    }
    {
        let resp = timestamp();
        println!("{:?}", resp);
    }
    {
        let resp = settings_get();
        println!("{:?}", resp);
    }
    {
        let resp = strategies_list();
        println!("{:?}", resp);
    }
    {
        let resp = strategies_list_my();
        println!("{:?}", resp);
    }
}

#[allow(unused)]
pub fn test_params() {
    // params
    {
        let resp = market("ethbtc");
        println!("{:?}", resp);
    }
    {
        let resp = ticker("ethbtc");
        println!("{:?}", resp);
    }
    {
        let resp = register_device("triage");
        println!("{:?}", resp);
    }
    {
        let resp = history(HashMap::new());
        println!("{:?}", resp);
    }
    {
        let resp = deposits(HashMap::new());
        println!("{:?}", resp);
    }
    {
        let resp = gen_deposit_address("btc");
        println!("{:?}", resp);
    }
    {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("market", "vrsc");
        let resp = orders_history(map);
        println!("{:?}", resp);
    }
    {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("fav-vrscbtc", "True");
        let resp = settings_store(map);
        println!("{:?}", resp);
    }
    {
        let resp = deposit_address("vrsc");
        println!("{:?}", resp);
    }
    {
        let resp = currency_info("btc");
        println!("{:?}", resp);
    }
    {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("currency", "eth");
        let resp = withdraws(map);
        println!("{:?}", resp);
    }
}

#[allow(unused)]
pub fn mutable_state_tests() {
    // requires further testing
    {
        let resp = deposit("fakestring");
        println!("{:?}", resp);
    }
    {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("market", "btcusd");
        let resp = orders_get(map);
        println!("{:?}", resp);
    }
    {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("test_field", "test_value");
        let resp = orders_post(map);
        println!("{:?}", resp);
    }
    {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("currency", "vrsc");
        map.insert("fund_uid", "address_here");
        map.insert("sum", "1.0000");
        let resp = create_withdraw(map);
        println!("{:?}", resp);
    }
    {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("currency", "btc");
        map.insert("uid", "address_here");
        map.insert("extra", "my_label");
        let resp = fund_source_create(map);
        println!("{:?}", resp);
    }
    {
        let resp = fund_source_remove("source_id_as_string");
        println!("{:?}", resp);
    }

}

