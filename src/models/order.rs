use chrono::NaiveDateTime;
use diesel::prelude::*;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::env;
use reqwest::header::{HeaderMap, AUTHORIZATION, HeaderValue, HeaderName};
use reqwest::Client;
use anyhow::Result;
use rust_decimal::Decimal;
use crate::schema::users::dsl::*;
use crate::schema::orders::dsl::*;
use diesel::result::Error as DieselError;
use crate::models::product::get_product_name_by_id;

use crate::models::user::{User, Claims, get_secret_key};

use super::product::Product;

#[derive(Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::orders)]
pub struct Order {
    pub id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub user_id: Uuid,
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

pub fn new_mollie_payment_line(conn: &mut PgConnection, product: Product, quantity: u32) -> MolliePaymentLine {
    let product_name = get_product_name_by_id(conn, product.id, "en_GB").unwrap();
    
    MolliePaymentLine {
        line_type: MolliePaymentLineType::Physical,
        description: String::from(product_name),
        quantity: quantity,
        quantity_unit: String::from("pcs"),
        unit_price: MollieAmount {
            currency: String::from("EUR"),
            value: product.price.to_string(),
        },
        total_amount: MollieAmount {
            currency: String::from("EUR"),
            value: (product.price * Decimal::from(quantity)).to_string(),
        },
        vat_rate: String::from("21.00"),
        vat_amount: MollieAmount {
            currency: String::from("EUR"),
            value: ((product.price * Decimal::from(quantity)) * Decimal::new(21, 2)).to_string(),
        },
        image_url: None,
        product_url: None,
    }
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
struct MolliePaymentResponse {
    // @todo
}

pub fn get_mollie_api_key() -> String {
    env::var("MOLLIE_API_KEY").expect("MOLLIE_API_KEY must be set")
}

pub async fn create_mollie_payment(form: MolliePaymentRequest) -> Result<MolliePaymentResponse> {
    // Create a new HTTP client
    let client = Client::new();

    let token = get_mollie_api_key();
    let auth_header_value = format!("Bearer {}", token);

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_header_value)?,
    );
    headers.insert(
        HeaderName::from_static("Accept"),
        HeaderValue::from_static("application/json"),
    );

    let response = client
        .post("https://api.mollie.com/v2/payments")
        .headers(headers)
        .json(&form)
        .send()
        .await?;

    let mollie_response = response
        .json()
        .await?;

    Ok(mollie_response)
}

impl Order {
    pub fn create_order(conn: &mut PgConnection, form: Order, token: &str) -> Result<Order, DieselError> {
        // Get the user id from the Bearer token
        let token = token.trim_start_matches("Bearer ").trim();
        let validation = Validation::new(Algorithm::HS256);
        let secret = get_secret_key();
        let decoded_token = decode::<Claims>(&token, &DecodingKey::from_secret(secret.as_ref()), &validation)
            .map_err(|_| diesel::result::Error::NotFound)?;

        let current_user_id: Uuid = Uuid::parse_str(&decoded_token.claims.sub).expect("Invalid UUID");

        let current_user = users
            .filter(id.eq(current_user_id))
            .first::<User>(conn);

        // Init lines of payment
        let mut payment_lines: Vec<MolliePaymentLine> = Vec::new();

        for item in &form.items {
            let new_line = new_mollie_payment_line(conn, item.product, item.quantity);
            payment_lines.push(new_line);
        }
        
        // Init a MolliePaymentRequest
        let new_payment_request = MolliePaymentRequest {
            description: String::from("Order n°"),
            amount: MollieAmount {
                currency: String::from("EUR"),
                value: String::from(""),
            },
            redirect_url: String::from(""),
            cancel_url: None,
            webhook_url: None,
            lines: Vec<MolliePaymentLine>,
            billing_address: Option<MollieAddress>,
            shipping_address: Option<MollieAddress>,
            locale: MollieLocale::EnGB,
        };

        // Create a new Order instance with the necessary fields populated
        let new_order = Order {
            id: Uuid::new_v4(),
            created_at: None,
            updated_at: None,
            user_id: current_user_id, // Assign user_id from the token
            payment_mode: form.payment_mode, // Assign payment_mode from the form
            mollie_payment_id: form.mollie_payment_id, // Assign Mollie payment ID from the form, if any
            mollie_payment_url: form.mollie_payment_url, // Assign Mollie payment URL from the form, if any
            status: form.status, // Assign status from the form, if any
        };

        // Insert the new order into the orders table
        diesel::insert_into(orders::table)
            .values(&new_order)
            .get_result(conn)
    }
}
