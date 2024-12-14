use crate::{error::Error, ex};

use sqlx::Row;

use super::Database;

pub struct SearchResult {
    pub id: i64,
    pub url: String,
    pub weight: f64,
    //pub matches: Vec<String>,
}

impl Database {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, Error> {
        let sql = r#"
        SELECT h.https, h.name, p.path, q.query, lk.link_id, SUM(lk.count / k.count) as weight, GROUP_CONCAT(k.name)
        FROM link_keyword lk
        LEFT JOIN link l ON l.id = lk.link_id
        LEFT JOIN keyword k ON k.id = lk.keyword_id
        LEFT JOIN host h ON h.id = l.host_id
        LEFT JOIN path p ON p.id = l.path_id
        LEFT JOIN query q ON q.id = l.query_id
        WHERE lk.keyword_id = k.id AND k.name = $1
        GROUP BY lk.link_id
        ORDER BY weight DESC
        "#;
        let result = ex!(sqlx::query(sql).bind(query).fetch_all(&self.pool).await);

        let result: Vec<SearchResult> = result
            .into_iter()
            .map(|r| {
                let host: String = r.get(1);
                let path: String = r.get(2);
                let query: String = r.get(3);
                let lid: i64 = r.get(4);
                let weight: f64 = r.get(5);
                //let matches: String = r.get(6);

                let mut url = format!(
                    "{}://{}{}",
                    if r.get(0) { "https" } else { "http" },
                    host,
                    path
                );

                if !query.is_empty() {
                    url.push('?');
                    url.push_str(&query);
                }

                SearchResult {
                    id: lid,
                    url,
                    weight,
                    //matches: matches.split(",").collect(),
                }
            })
            .collect();

        Ok(result)
    }
}
