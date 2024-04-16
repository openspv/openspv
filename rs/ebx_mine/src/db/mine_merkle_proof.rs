use ebx_lib::merkle_proof::MerkleProof;
use sqlx::{types::chrono, MySqlPool};

#[derive(Debug, sqlx::FromRow)]
pub struct MineMerkleProof {
    pub merkle_root: String,
    pub tx_id: String,
    pub merkle_proof: String,
    pub created_at: chrono::NaiveDateTime,
}

impl MineMerkleProof {
    pub fn new(
        merkle_root: String,
        tx_id: String,
        merkle_proof: String,
        created_at: chrono::NaiveDateTime,
    ) -> Self {
        Self {
            merkle_root,
            tx_id,
            merkle_proof,
            created_at,
        }
    }

    pub fn from_merkle_proof(merkle_proof: &MerkleProof, tx_id: Vec<u8>) -> Self {
        Self {
            merkle_root: hex::encode(merkle_proof.root),
            tx_id: hex::encode(tx_id),
            merkle_proof: hex::encode(merkle_proof.to_u8_vec()),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    pub fn to_merkle_proof(&self) -> MerkleProof {
        MerkleProof::from_u8_vec(&hex::decode(&self.merkle_proof).unwrap())
    }

    pub async fn get(
        merkle_root: Vec<u8>,
        tx_id: Vec<u8>,
        pool: &MySqlPool,
    ) -> Result<Self, sqlx::Error> {
        let result = sqlx::query_as::<_, Self>(
            "SELECT * FROM mine_merkle_proof WHERE merkle_root = ? AND tx_id = ?",
        )
        .bind(merkle_root)
        .bind(tx_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    pub async fn upsert(&self, pool: &MySqlPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO mine_merkle_proof (merkle_root, tx_id, merkle_proof)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE
                merkle_proof = VALUES(merkle_proof)
            "#,
        )
        .bind(&self.merkle_root)
        .bind(&self.tx_id)
        .bind(&self.merkle_proof)
        .execute(pool)
        .await?;
        Ok(())
    }
}