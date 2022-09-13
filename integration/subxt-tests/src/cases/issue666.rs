use contract_transcode::ContractMessageTranscoder;
use parity_scale_codec::Encode;

use crate::{load_project, DeployContract, Execution, WriteContract, API};

#[tokio::test]
async fn case() -> anyhow::Result<()> {
    let api = API::new().await?;

    let flipper_code = std::fs::read("./contracts/Flip.wasm")?;
    let inc_code = std::fs::read("./contracts/Inc.wasm")?;

    let p_flipper = load_project("./contracts/Flip.contract")?;
    let t_flipper = ContractMessageTranscoder::new(&p_flipper);

    let p_inc = load_project("./contracts/Inc.contract")?;
    let t_inc = ContractMessageTranscoder::new(&p_inc);

    let selector = t_flipper.encode::<_, String>("new", ["true".into()])?;

    let flipper = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        code: flipper_code,
    }
    .execute(&api)
    .await?;

    // flip on Flip
    let selector = t_flipper.encode::<_, String>("flip", [])?;

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: flipper.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    let selector = t_inc.encode::<_, String>(
        "new",
        [format!(
            "0x{}",
            hex::encode(flipper.contract_address.clone())
        )],
    )?;

    let inc = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        code: inc_code,
    }
    .execute(&api)
    .await?;

    // superFlip on Inc
    let selector = t_inc.encode::<_, String>("superFlip", [])?;

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: inc.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    Ok(())
}
