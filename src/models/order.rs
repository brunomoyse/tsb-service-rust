use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::env;

use crate::schema::orders;

#[derive(Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::orders)]
pub struct Order {
    pub id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub user_id: Option<String>,
    pub payment_mode: Option<String>,
    pub mollie_payment_id: Option<String>,
    pub mollie_payment_url: Option<String>,
    pub status: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MollieAmount {
    pub currency: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MolliePaymentLineType {
    Physical,
    Digital,
    ShippingFee,
    Discount,
    StoreCredit,
    GiftCard,
    Surcharge,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MolliePaymentLine {
    #[serde(rename = "type")]  // This tells serde to serialize/deserialize the field as "type" in JSON
    pub line_type: MolliePaymentLineType,
    pub description: String,
    pub quantity: u32,
    pub quantity_unit: String,
    pub unit_price: MollieAmount,
    pub total_amount: MollieAmount,
    pub vat_rate: String,
    pub vat_amount: MollieAmount,
    pub image_url: Option<String>,
    pub product_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MollieAddress {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub organization_name: Option<String>,
    pub street_and_number: Option<String>,
    pub street_additional: Option<String>,
    pub postal_code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MollieLocale {
    #[serde(rename = "en_GB")]
    EnGB,

    #[serde(rename = "nl_BE")]
    NlBE,

    #[serde(rename = "fr_BE")]
    FrBE,
}

pub struct MolliePaymentRequest {
    pub description: String,
    pub amount: MollieAmount,
    pub redirect_url: String,
    pub cancel_url: Option<String>,
    pub webhook_url: Option<String>,
    pub lines: Vec<MolliePaymentLine>,
    pub billing_address: Option<MollieAddress>,
    pub shipping_address: Option<MollieAddress>,
    pub locale: MollieLocale,
}

pub fn get_bearer_token() -> String {
    env::var("MOLLIE_API_KEY").expect("MOLLIE_API_KEY must be set")
}

pub fn create_mollie_payment(form: MolliePaymentRequest) -> Result<MolliePaymentResponse> {
    let client = reqwest::blocking::Client::new();
    let token = get_bearer_token();
    let response = client
        .post("https://api.mollie.com/v2/payments")
        .header("Authorization", format!("Bearer {}", token))
        .json(&form)
        .send()
        .expect("Failed to send request to Mollie API");

    let mollie_response: MolliePaymentResponse = response
        .json()
        .with_context(|| "Failed to parse JSON response from Mollie API")?;

    Ok(mollie_response)
}

impl Order {


    pub fn create_product(conn: &mut PgConnection, form: Order) -> Result<Order> {
        // Create a new Order instance with the necessary fields populated
        let new_order = Order {
            id: Uuid::new_v4(), // Generate a new UUID for the order
            created_at: None, // Set created_at to None
            updated_at: None, // Set updated_at to None
            user_id: form.user_id, // Assign user_id from the form
            payment_mode: form.payment_mode, // Assign payment_mode from the form
            mollie_payment_id: form.mollie_payment_id, // Assign Mollie payment ID from the form, if any
            mollie_payment_url: form.mollie_payment_url, // Assign Mollie payment URL from the form, if any
            status: form.status, // Assign status from the form, if any
        };
    
        // Insert the new order into the orders table
        diesel::insert_into(orders::table)
            .values(&new_order)
            .get_result(conn)
            .map_err(|e| anyhow::anyhow!("Failed to create order: {}", e))
    }
}
