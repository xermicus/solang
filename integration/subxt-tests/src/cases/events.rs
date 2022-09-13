use contract_transcode::ContractMessageTranscoder;
use parity_scale_codec::{Compact, Decode, Input};
use sp_core::{crypto::AccountId32, hexdisplay::AsBytesRef};

use crate::{load_project, DeployContract, Execution, WriteContract, API};
use hex::FromHex;

#[tokio::test]
async fn case() -> anyhow::Result<()> {
    let api = API::new().await?;
    let code = std::fs::read("./contracts/events.wasm")?;

    let p = load_project("./contracts/events.contract")?;
    let transcoder = ContractMessageTranscoder::new(&p);

    let selector = transcoder.encode::<_, String>("new", [])?;

    let deployed = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        code,
    }
    .execute(&api)
    .await?;

    let selector = transcoder.encode::<_, String>("emit_event", [])?;

    let rs = WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: deployed.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    assert_eq!(rs.events.len(), 2);

    // TODO: currently event decoding is different than ink, as we can see in contract-transcode.

    let e1 = &rs.events[0];

    let e1_buffer = &mut e1.data.as_slice();

    let topic = e1_buffer.read_byte()?;
    assert_eq!(topic, 0);

    // mimic the solidity struct type
    #[derive(Decode)]
    struct Foo1 {
        id: i64,
        s: String,
    }

    let Foo1 { id, s } = Foo1::decode(e1_buffer)?;
    assert_eq!((id, s.as_str()), (254, "hello there"));

    let e2 = &rs.events[1];
    let e2_buffer = &mut e2.data.as_slice();

    let topic = e2_buffer.read_byte()?;
    assert_eq!(topic, 1);

    // mimic the solidity struct type
    #[derive(Decode)]
    struct Foo2 {
        id: i64,
        s2: String,
        a: AccountId32,
    }

    let Foo2 { id, s2, a } = Foo2::decode(e2_buffer)?;
    assert_eq!(
        (id, s2.as_str(), a),
        (
            i64::from_str_radix("7fffffffffffffff", 16)?,
            "minor",
            deployed.contract_address
        )
    );

    Ok(())
}
