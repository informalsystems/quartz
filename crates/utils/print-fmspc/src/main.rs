use dcap_qvl::collateral::get_collateral;

const DEFAULT_PCCS_URL: &str = "https://localhost:8081/sgx/certification/v4/";

#[tokio::main]
async fn main() {
    let pccs_url = std::env::var("PCCS_URL").unwrap_or_else(|_| DEFAULT_PCCS_URL.to_string());
    let quote = {
        let quote_hex = std::env::var("QUOTE").expect("QUOTE is not found");
        hex::decode(quote_hex).expect("QUOTE is not valid hex")
    };

    let collateral = get_collateral(&pccs_url, &quote, std::time::Duration::from_secs(10))
        .await
        .expect("failed to get collateral");
    let tcb_info: serde_json::Value =
        serde_json::from_str(&collateral.tcb_info).expect("Retrieved Tcbinfo is not valid JSON");

    eprintln!("{}", tcb_info["fmspc"]);
}
