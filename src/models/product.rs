use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;
use std::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::schema::{product_categories, product_category_translations, product_product_category, product_translations, products};

#[derive(Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::products)]
pub struct Product {
    pub id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub price: Option<f64>,
    pub is_active: bool,
    pub code: Option<String>,
    pub slug: Option<String>
}

#[derive(Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::product_translations)]
pub struct ProductTranslation {
    pub id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub name: String,
    pub description: Option<String>,
    pub locale: String,
    pub product_id: Uuid
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct ProductInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub code: Option<String>,
    pub slug: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct CategoryWithProducts {
    pub id: Uuid,
    pub name: String,
    pub order: Option<i32>,
    pub products: Vec<ProductInfo>,
}

impl Product {
    pub fn get_products_grouped_by_category(conn: &mut PgConnection, locale: &str, search_query: Option<&str>) -> Result<Vec<CategoryWithProducts>, Box<dyn Error + Send + Sync>> {
        let mut query = product_categories::table
            .inner_join(product_category_translations::table.on(product_categories::id.eq(product_category_translations::product_category_id)))
            .inner_join(product_product_category::table.on(product_categories::id.eq(product_product_category::product_category_id)))
            .inner_join(products::table.on(product_product_category::product_id.eq(products::id)))
            .inner_join(product_translations::table.on(products::id.eq(product_translations::product_id)))
            .filter(product_translations::locale.eq(locale))
            .filter(product_category_translations::locale.eq(locale))
            .filter(products::is_active.eq(true))
            .select((
                product_categories::id,
                product_category_translations::name,
                product_categories::order,
                products::id,
                product_translations::name,
                product_translations::description,
                products::price,
                products::code,
                products::slug,
            ))
            .into_boxed();

        if let Some(query_str) = search_query {
            let terms: Vec<&str> = query_str.split_whitespace().collect();
            for term in terms {
                query = query.filter(
                    product_translations::name.ilike(format!("%{}%", term))
                    .or(product_category_translations::name.ilike(format!("%{}%", term)))
                );
            }
        }

        let raw_data = query.load::<(Uuid, String, Option<i32>, Uuid, String, Option<String>, Option<f64>, Option<String>, Option<String>)>(conn)?;

        let mut categories: HashMap<Uuid, CategoryWithProducts> = HashMap::new();
        for (id, name, order, product_id, product_name, description, price, code, slug) in raw_data {
            let category = categories.entry(id).or_insert_with(|| CategoryWithProducts {
                id,
                name,
                order,
                products: vec![],
            });
            category.products.push(ProductInfo {
                id: product_id,
                name: product_name,
                description,
                price,
                code,
                slug,
            });
        }

        let mut res: Vec<CategoryWithProducts> = categories.into_values().collect();
        // Sort categories
        res.sort_by_key(|c| c.order);
        // Sort products by name
        res.iter_mut().for_each(|c| c.products.sort_by_key(|p| p.name.clone()));

        Ok(res)
    }
}
