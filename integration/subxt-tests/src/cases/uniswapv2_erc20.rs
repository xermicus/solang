use contract_transcode::ContractMessageTranscoder;
use hex::FromHex;
use ink_metadata::InkProject;
use parity_scale_codec::{Decode, DecodeAll, Encode, Input};
use rand::Rng;
use sp_core::{crypto::AccountId32, hexdisplay::AsBytesRef, keccak_256, U256};
use subxt::ext::bitvec::macros::internal::funty::Numeric;

use crate::{load_project, DeployContract, Execution, ReadContract, WriteContract, API};

#[tokio::test]
async fn setup() -> anyhow::Result<()> {
    let api = API::new().await?;

    let w = MockWorld::init(&api).await?;

    let transcoder = ContractMessageTranscoder::new(&w.project);

    let selector = transcoder.encode::<_, String>("name", [])?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        contract_address: w.token_addr.clone(),
    }
    .execute(&api)
    .await
    .and_then(|v| <String>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(rs, "Uniswap V2");

    let selector = transcoder.encode::<_, String>("symbol", [])?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        contract_address: w.token_addr.clone(),
    }
    .execute(&api)
    .await
    .and_then(|v| <String>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(rs, "UNI-V2");

    let selector = transcoder.encode::<_, String>("decimals", [])?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        contract_address: w.token_addr.clone(),
    }
    .execute(&api)
    .await
    .and_then(|v| <u8>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(rs, 18);

    let selector = transcoder.encode::<_, String>("totalSupply", [])?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        contract_address: w.token_addr.clone(),
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(
        rs,
        U256::from_dec_str("10000")? * U256::from(10).pow(18_u8.into())
    );

    let selector =
        transcoder.encode::<_, String>("balanceOf", [format!("0x{}", hex::encode(&w.alice))])?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        contract_address: w.token_addr.clone(),
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(
        rs,
        U256::from_dec_str("10000")? * U256::from(10).pow(18_u8.into())
    );

    let selector = transcoder.encode::<_, String>("DOMAIN_SEPARATOR", [])?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        contract_address: w.token_addr.clone(),
    }
    .execute(&api)
    .await
    .and_then(|v| <[u8; 32]>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    let expected = [
        keccak_256(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
                .as_bytes(),
        )
        .to_vec(),
        keccak_256("Uniswap V2".as_bytes()).to_vec(),
        keccak_256("1".as_bytes()).to_vec(),
        hex::decode("0100000000000000000000000000000000000000000000000000000000000000")?,
        AsRef::<[u8; 32]>::as_ref(&w.token_addr).to_vec(),
    ]
    .concat();

    let expected = keccak_256(&expected[..]);
    assert_eq!(rs, expected);

    let selector = transcoder.encode::<_, String>("PERMIT_TYPEHASH", [])?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        contract_address: w.token_addr.clone(),
    }
    .execute(&api)
    .await
    .and_then(|v| <[u8; 32]>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(
        rs,
        <[u8; 32]>::from_hex("6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9")?
    );

    Ok(())
}

struct MockWorld {
    alice: AccountId32,
    dave: AccountId32,
    token_addr: AccountId32,
    project: InkProject,
}

#[tokio::test]
async fn approve() -> anyhow::Result<()> {
    let api = API::new().await?;

    let w = MockWorld::init(&api).await?;

    let transcoder = ContractMessageTranscoder::new(&w.project);

    let mut selector = transcoder.encode("approve", [format!("0x{}", hex::encode(&w.dave))])?;

    U256::from(10_u128.pow(18)).encode_to(&mut selector);

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    let selector = transcoder.encode(
        "allowance",
        [
            format!("0x{}", hex::encode(&w.alice)),
            format!("0x{}", hex::encode(&w.dave)),
        ],
    )?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(rs, U256::from(10_u128.pow(18)));

    Ok(())
}

#[tokio::test]
async fn transfer() -> anyhow::Result<()> {
    let api = API::new().await?;

    let w = MockWorld::init(&api).await?;

    let transcoder = ContractMessageTranscoder::new(&w.project);

    let mut selector = transcoder.encode("transfer", [format!("0x{}", hex::encode(&w.dave))])?;
    U256::from(10_u128.pow(18)).encode_to(&mut selector);

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    let selector = transcoder.encode("balanceOf", [format!("0x{}", hex::encode(&w.alice))])?;

    let alice_balance = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(
        alice_balance,
        U256::from_dec_str("10000")? * U256::from(10).pow(18_u8.into())
            - U256::from(10_u128.pow(18))
    );

    let selector = transcoder.encode("balanceOf", [format!("0x{}", hex::encode(&w.dave))])?;

    let dave_balance = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(
        alice_balance,
        U256::from_dec_str("10000")? * U256::from(10).pow(18_u8.into())
            - U256::from(10_u128.pow(18))
    );

    assert_eq!(dave_balance, U256::from(10_u128.pow(18)));

    Ok(())
}

#[tokio::test]
async fn transfer_from() -> anyhow::Result<()> {
    let api = API::new().await?;

    let w = MockWorld::init(&api).await?;

    let transcoder = ContractMessageTranscoder::new(&w.project);

    let mut selector = transcoder.encode("approve", [format!("0x{}", hex::encode(&w.dave))])?;
    U256::from(10_u128.pow(18)).encode_to(&mut selector);

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    let mut selector = transcoder.encode(
        "transferFrom",
        [
            format!("0x{}", hex::encode(&w.alice)),
            format!("0x{}", hex::encode(&w.dave)),
        ],
    )?;
    U256::from(10_u128.pow(18)).encode_to(&mut selector);

    WriteContract {
        caller: sp_keyring::AccountKeyring::Dave,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    let selector = transcoder.encode(
        "allowance",
        [
            format!("0x{}", hex::encode(&w.alice)),
            format!("0x{}", hex::encode(&w.dave)),
        ],
    )?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(rs, 0_u8.into());

    let selector = transcoder.encode("balanceOf", [format!("0x{}", hex::encode(&w.alice))])?;

    let alice_balance = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(
        alice_balance,
        U256::from_dec_str("10000")? * U256::from(10).pow(18_u8.into())
            - U256::from(10_u128.pow(18))
    );

    let selector = transcoder.encode("balanceOf", [format!("0x{}", hex::encode(&w.dave))])?;

    let dave_balance = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(
        alice_balance,
        U256::from_dec_str("10000")? * U256::from(10).pow(18_u8.into())
            - U256::from(10_u128.pow(18))
    );

    assert_eq!(dave_balance, U256::from(10_u128.pow(18)));

    Ok(())
}

#[tokio::test]
async fn transfer_from_max() -> anyhow::Result<()> {
    let api = API::new().await?;

    let w = MockWorld::init(&api).await?;

    let transcoder = ContractMessageTranscoder::new(&w.project);

    let mut selector = transcoder.encode("approve", [format!("0x{}", hex::encode(&w.dave))])?;
    U256::MAX.encode_to(&mut selector);

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    let mut selector = transcoder.encode(
        "transferFrom",
        [
            format!("0x{}", hex::encode(&w.alice)),
            format!("0x{}", hex::encode(&w.dave)),
        ],
    )?;
    U256::from(10).pow(18_u8.into()).encode_to(&mut selector);

    WriteContract {
        caller: sp_keyring::AccountKeyring::Dave,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    let selector = transcoder.encode(
        "allowance",
        [
            format!("0x{}", hex::encode(&w.alice)),
            format!("0x{}", hex::encode(&w.dave)),
        ],
    )?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(rs, U256::MAX);

    let selector = transcoder.encode("balanceOf", [format!("0x{}", hex::encode(&w.alice))])?;

    let alice_balance = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(
        alice_balance,
        U256::from_dec_str("10000")? * U256::from(10).pow(18_u8.into())
            - U256::from(10_u128.pow(18))
    );

    let selector = transcoder.encode("balanceOf", [format!("0x{}", hex::encode(&w.dave))])?;

    let dave_balance = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: w.token_addr.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <U256>::decode(&mut &v.return_value[..]).map_err(Into::into))?;

    assert_eq!(
        alice_balance,
        U256::from_dec_str("10000")? * U256::from(10).pow(18_u8.into())
            - U256::from(10_u128.pow(18))
    );

    assert_eq!(dave_balance, U256::from(10_u128.pow(18)));

    Ok(())
}

impl MockWorld {
    async fn init(api: &API) -> anyhow::Result<Self> {
        let alice: AccountId32 = sp_keyring::AccountKeyring::Alice.to_account_id();
        let dave: AccountId32 = sp_keyring::AccountKeyring::Dave.to_account_id();
        let code = std::fs::read("./contracts/ERC20.wasm")?;

        let p = load_project("./contracts/ERC20.contract")?;

        let transcoder = ContractMessageTranscoder::new(&p);

        let mut selector = transcoder.encode::<_, String>("new", [])?;

        (U256::from_dec_str("10000")? * U256::from(10).pow(18_u8.into())).encode_to(&mut selector);

        let contract = DeployContract {
            caller: sp_keyring::AccountKeyring::Alice,
            selector,
            value: 0,
            code,
        }
        .execute(api)
        .await?;

        Ok(Self {
            alice,
            dave,
            token_addr: contract.contract_address,
            project: p,
        })
    }
}
