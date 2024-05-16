use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

#[cw_serde]
pub struct IASReport {
    pub report: ReportBody,
    #[serde(rename = "reportsig")]
    pub report_sig: Binary,
}

#[cw_serde]
pub struct ReportBody {
    pub id: String,
    pub timestamp: String,
    pub version: u64,
    #[serde(rename = "epidPseudonym")]
    pub epid_pseudonym: Binary,
    #[serde(rename = "advisoryURL")]
    pub advisory_url: String,
    #[serde(rename = "advisoryIDs")]
    pub advisory_ids: Vec<String>,
    #[serde(rename = "isvEnclaveQuoteStatus")]
    pub isv_enclave_quote_status: String,
    #[serde(rename = "platformInfoBlob")]
    pub platform_info_blob: String,
    #[serde(rename = "isvEnclaveQuoteBody")]
    pub isv_enclave_quote_body: IsvEnclaveQuoteBody,
}

#[cw_serde]
#[serde(transparent)]
pub struct IsvEnclaveQuoteBody(Binary);

impl IsvEnclaveQuoteBody {
    pub fn mrenclave(&self) -> [u8; 32] {
        Self::array_chunk(self.0.as_slice(), 112)
    }

    pub fn user_data(&self) -> [u8; 64] {
        Self::array_chunk(self.0.as_slice(), 368)
    }

    fn array_chunk<const N: usize>(quote_body: &[u8], offset: usize) -> [u8; N] {
        assert!(offset + N <= quote_body.len());
        quote_body[offset..offset + N]
            .try_into()
            .expect("array length mismatch")
    }
}
