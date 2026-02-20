use crate::{Subscription, SubscriptionStatus, SubscriptionVault, SubscriptionVaultClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

/// Helper: register contract, init, and return client + reusable addresses.
fn setup_env() -> (Env, SubscriptionVaultClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SubscriptionVault, ());
    let client = SubscriptionVaultClient::new(&env, &contract_id);

    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    client.init(&token, &admin);

    (env, client, token, admin)
}

/// Helper: create a subscription for a given subscriber+merchant and return its id.
fn create_sub(
    env: &Env,
    client: &SubscriptionVaultClient,
    subscriber: &Address,
    merchant: &Address,
    amount: i128,
) -> u32 {
    client.create_subscription(
        subscriber,
        merchant,
        &amount,
        &(30u64 * 24 * 60 * 60), // 30 days
        &false,
    )
}

// ─── Existing tests ───────────────────────────────────────────────────────────

#[test]
fn test_init_and_struct() {
    let env = Env::default();
    let contract_id = env.register(SubscriptionVault, ());
    let client = SubscriptionVaultClient::new(&env, &contract_id);

    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    client.init(&token, &admin);
}

#[test]
fn test_subscription_struct() {
    let env = Env::default();
    let sub = Subscription {
        subscriber: Address::generate(&env),
        merchant: Address::generate(&env),
        amount: 10_000_0000,
        interval_seconds: 30 * 24 * 60 * 60,
        last_payment_timestamp: 0,
        status: SubscriptionStatus::Active,
        prepaid_balance: 50_000_0000,
        usage_enabled: false,
    };
    assert_eq!(sub.status, SubscriptionStatus::Active);
}

// ─── Merchant view helper tests ───────────────────────────────────────────────

#[test]
fn test_merchant_with_no_subscriptions() {
    let (env, client, _, _) = setup_env();
    let merchant = Address::generate(&env);

    let subs = client.get_subscriptions_by_merchant(&merchant, &0, &10);
    assert_eq!(subs.len(), 0);

    let count = client.get_merchant_subscription_count(&merchant);
    assert_eq!(count, 0);
}

#[test]
fn test_merchant_with_one_subscription() {
    let (env, client, _, _) = setup_env();
    let subscriber = Address::generate(&env);
    let merchant = Address::generate(&env);

    let id = create_sub(&env, &client, &subscriber, &merchant, 10_000_000);

    let subs = client.get_subscriptions_by_merchant(&merchant, &0, &10);
    assert_eq!(subs.len(), 1);

    let sub = subs.get(0).unwrap();
    assert_eq!(sub.subscriber, subscriber);
    assert_eq!(sub.merchant, merchant);
    assert_eq!(sub.amount, 10_000_000);
    assert_eq!(sub.status, SubscriptionStatus::Active);

    // Verify get_subscription returns the same data
    let by_id = client.get_subscription(&id);
    assert_eq!(by_id.subscriber, subscriber);

    assert_eq!(client.get_merchant_subscription_count(&merchant), 1);
}

#[test]
fn test_merchant_with_multiple_subscriptions() {
    let (env, client, _, _) = setup_env();
    let merchant = Address::generate(&env);

    let sub1 = Address::generate(&env);
    let sub2 = Address::generate(&env);
    let sub3 = Address::generate(&env);

    create_sub(&env, &client, &sub1, &merchant, 5_000_000);
    create_sub(&env, &client, &sub2, &merchant, 10_000_000);
    create_sub(&env, &client, &sub3, &merchant, 20_000_000);

    let subs = client.get_subscriptions_by_merchant(&merchant, &0, &10);
    assert_eq!(subs.len(), 3);

    // Verify chronological (insertion) order
    assert_eq!(subs.get(0).unwrap().amount, 5_000_000);
    assert_eq!(subs.get(1).unwrap().amount, 10_000_000);
    assert_eq!(subs.get(2).unwrap().amount, 20_000_000);

    assert_eq!(client.get_merchant_subscription_count(&merchant), 3);
}

#[test]
fn test_pagination_basic() {
    let (env, client, _, _) = setup_env();
    let merchant = Address::generate(&env);

    // Create 5 subscriptions
    for i in 0..5 {
        let subscriber = Address::generate(&env);
        create_sub(&env, &client, &subscriber, &merchant, (i + 1) * 1_000_000);
    }

    // Request first 2
    let page = client.get_subscriptions_by_merchant(&merchant, &0, &2);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().amount, 1_000_000);
    assert_eq!(page.get(1).unwrap().amount, 2_000_000);
}

#[test]
fn test_pagination_offset() {
    let (env, client, _, _) = setup_env();
    let merchant = Address::generate(&env);

    for i in 0..5 {
        let subscriber = Address::generate(&env);
        create_sub(&env, &client, &subscriber, &merchant, (i + 1) * 1_000_000);
    }

    // Request 2 starting from offset 2
    let page = client.get_subscriptions_by_merchant(&merchant, &2, &2);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().amount, 3_000_000);
    assert_eq!(page.get(1).unwrap().amount, 4_000_000);
}

#[test]
fn test_pagination_beyond_end() {
    let (env, client, _, _) = setup_env();
    let merchant = Address::generate(&env);

    for i in 0..5 {
        let subscriber = Address::generate(&env);
        create_sub(&env, &client, &subscriber, &merchant, (i + 1) * 1_000_000);
    }

    // Request 10 starting from offset 3 → should return only last 2
    let page = client.get_subscriptions_by_merchant(&merchant, &3, &10);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().amount, 4_000_000);
    assert_eq!(page.get(1).unwrap().amount, 5_000_000);
}

#[test]
fn test_pagination_start_past_end() {
    let (env, client, _, _) = setup_env();
    let merchant = Address::generate(&env);

    let subscriber = Address::generate(&env);
    create_sub(&env, &client, &subscriber, &merchant, 1_000_000);

    // Start way past the end
    let page = client.get_subscriptions_by_merchant(&merchant, &100, &10);
    assert_eq!(page.len(), 0);
}

#[test]
fn test_multiple_merchants_isolated() {
    let (env, client, _, _) = setup_env();
    let merchant_a = Address::generate(&env);
    let merchant_b = Address::generate(&env);

    let sub1 = Address::generate(&env);
    let sub2 = Address::generate(&env);
    let sub3 = Address::generate(&env);

    create_sub(&env, &client, &sub1, &merchant_a, 1_000_000);
    create_sub(&env, &client, &sub2, &merchant_a, 2_000_000);
    create_sub(&env, &client, &sub3, &merchant_b, 9_000_000);

    // Merchant A sees only their 2 subscriptions
    let a_subs = client.get_subscriptions_by_merchant(&merchant_a, &0, &10);
    assert_eq!(a_subs.len(), 2);
    assert_eq!(a_subs.get(0).unwrap().amount, 1_000_000);
    assert_eq!(a_subs.get(1).unwrap().amount, 2_000_000);

    // Merchant B sees only their 1 subscription
    let b_subs = client.get_subscriptions_by_merchant(&merchant_b, &0, &10);
    assert_eq!(b_subs.len(), 1);
    assert_eq!(b_subs.get(0).unwrap().amount, 9_000_000);

    assert_eq!(client.get_merchant_subscription_count(&merchant_a), 2);
    assert_eq!(client.get_merchant_subscription_count(&merchant_b), 1);
}

#[test]
fn test_merchant_subscription_count() {
    let (env, client, _, _) = setup_env();
    let merchant = Address::generate(&env);

    assert_eq!(client.get_merchant_subscription_count(&merchant), 0);

    for _ in 0..4 {
        let subscriber = Address::generate(&env);
        create_sub(&env, &client, &subscriber, &merchant, 5_000_000);
    }

    assert_eq!(client.get_merchant_subscription_count(&merchant), 4);
}
