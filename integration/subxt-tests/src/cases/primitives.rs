use std::{ops::BitOr, str::FromStr};

use contract_transcode::ContractMessageTranscoder;
use hex::FromHex;
use num_bigint::{BigInt, BigUint, Sign};
use parity_scale_codec::{Decode, Encode, Input};
use sp_core::{crypto::AccountId32, hexdisplay::AsBytesRef, H256, U256};
use subxt::ext::sp_runtime::{traits::One, MultiAddress};

use crate::{load_project, DeployContract, Execution, ReadContract, WriteContract, API};

async fn query<T: Decode>(api: &API, addr: &AccountId32, selector: &[u8]) -> anyhow::Result<T> {
    ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: addr.clone(),
        value: 0,
        selector: selector.to_vec(),
    }
    .execute(api)
    .await
    .and_then(|v| T::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))
}

#[tokio::test]
async fn case() -> anyhow::Result<()> {
    let api = API::new().await?;
    let code = std::fs::read("./contracts/primitives.wasm")?;

    let p = load_project("./contracts/primitives.contract")?;

    let transcoder = ContractMessageTranscoder::new(&p);

    let selector = transcoder.encode::<_, String>("new", [])?;

    let contract = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        code,
    }
    .execute(&api)
    .await?;

    // test res
    #[derive(Encode, Decode)]
    enum oper {
        add,
        sub,
        mul,
        div,
        r#mod,
        pow,
        shl,
        shr,
        or,
        and,
        xor,
    }

    let selector = transcoder.encode::<_, String>("is_mul", ["mul".into()])?;

    let is_mul = query::<bool>(&api, &contract.contract_address, &selector).await?;
    assert!(is_mul);

    let selector = transcoder.encode::<_, String>("return_div", [])?;

    let return_div = query::<oper>(&api, &contract.contract_address, &selector).await?;
    if let oper::div = return_div {
    } else {
        panic!("not div");
    }

    let selector = transcoder.encode("op_i64", ["add", "1000", "4100"])?;

    let res = query::<i64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 5100);

    let selector = transcoder.encode("op_i64", ["sub", "1000", "4100"])?;

    let res = query::<i64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, -3100);

    let selector = transcoder.encode("op_i64", ["mul", "1000", "4100"])?;

    let res = query::<i64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 4100000);

    let selector = transcoder.encode("op_i64", ["div", "1000", "10"])?;

    let res = query::<i64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 100);

    let selector = transcoder.encode("op_i64", ["mod", "1000", "99"])?;

    let res = query::<i64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 10);

    let selector = transcoder.encode("op_i64", ["shl", "-1000", "8"])?;

    let res = query::<i64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, -256000);

    let selector = transcoder.encode("op_i64", ["shr", "-1000", "8"])?;

    let res = query::<i64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, -4);

    // op_u64

    let selector = transcoder.encode("op_u64", ["add", "1000", "4100"])?;

    let res = query::<u64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 5100);

    let selector = transcoder.encode("op_u64", ["sub", "1000", "4100"])?;

    let res = query::<u64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 18446744073709548516);

    let selector = transcoder.encode("op_u64", ["mul", "123456789", "123456789"])?;

    let res = query::<u64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 15241578750190521);

    let selector = transcoder.encode("op_u64", ["div", "123456789", "100"])?;

    let res = query::<u64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 1234567);

    let selector = transcoder.encode("op_u64", ["mod", "123456789", "100"])?;

    let res = query::<u64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 89);

    let selector = transcoder.encode("op_u64", ["pow", "3", "7"])?;

    let res = query::<u64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 2187);

    let selector = transcoder.encode("op_u64", ["shl", "1000", "8"])?;

    let res = query::<u64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 256000);

    let selector = transcoder.encode("op_u64", ["shr", "1000", "8"])?;

    let res = query::<u64>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, 3);

    // op_i256
    // TODO: currently contract-transcode don't support encoding/decoding of I256 type so we'll need  to encode it manually
    let mut selector = transcoder.encode("op_i256", ["add"])?;
    U256::from_dec_str("1000")?.encode_to(&mut selector);
    U256::from_dec_str("4100")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, U256::from(5100_u128));

    let mut selector = transcoder.encode("op_i256", ["sub"])?;
    U256::from_dec_str("1000")?.encode_to(&mut selector);
    U256::from_dec_str("4100")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    // use two's compliment to get negative value in
    assert_eq!(res, !U256::from(3100_u128) + U256::one());

    let mut selector = transcoder.encode("op_i256", ["mul"])?;
    U256::from_dec_str("1000")?.encode_to(&mut selector);
    U256::from_dec_str("4100")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, U256::from_dec_str("4100000")?);

    let mut selector = transcoder.encode("op_i256", ["div"])?;
    U256::from_dec_str("1000")?.encode_to(&mut selector);
    U256::from_dec_str("10")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, U256::from_dec_str("100")?);

    let mut selector = transcoder.encode("op_i256", ["mod"])?;
    U256::from_dec_str("1000")?.encode_to(&mut selector);
    U256::from_dec_str("99")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, U256::from_dec_str("10")?);

    let mut selector = transcoder.encode("op_i256", ["shl"])?;
    (!U256::from_dec_str("10000000000000")? + U256::one()).encode_to(&mut selector);
    U256::from_dec_str("8")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, !U256::from_dec_str("2560000000000000")? + U256::one());

    let mut selector = transcoder.encode("op_i256", ["shr"])?;
    (!U256::from_dec_str("10000000000000")? + U256::one()).encode_to(&mut selector);
    U256::from_dec_str("8")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;
    assert_eq!(res, !U256::from(39062500000_i64) + U256::one());

    // op_u256
    // TODO: currently U256 from string is not supported by contract-transcode, we'll need to encode it manually
    let mut selector = transcoder.encode("op_u256", ["add"])?;
    U256::from_dec_str("1000")?.encode_to(&mut selector);
    U256::from_dec_str("4100")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, U256::from(5100_u128));

    let mut selector = transcoder.encode("op_u256", ["sub"])?;
    U256::from_dec_str("1000")?.encode_to(&mut selector);
    U256::from_dec_str("4100")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, !U256::from(3100_u128) + U256::one());

    let mut selector = transcoder.encode("op_u256", ["mul"])?;
    U256::from_dec_str("123456789")?.encode_to(&mut selector);
    U256::from_dec_str("123456789")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, U256::from(15241578750190521_u128));

    let mut selector = transcoder.encode("op_u256", ["div"])?;
    U256::from_dec_str("123456789")?.encode_to(&mut selector);
    U256::from_dec_str("100")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, U256::from(1234567_u128));

    let mut selector = transcoder.encode("op_u256", ["mod"])?;
    U256::from_dec_str("123456789")?.encode_to(&mut selector);
    U256::from_dec_str("100")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, U256::from(89_u64));

    let mut selector = transcoder.encode("op_u256", ["pow"])?;
    U256::from_dec_str("123456789")?.encode_to(&mut selector);
    U256::from_dec_str("9")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(
        res.to_string(),
        "6662462759719942007440037531362779472290810125440036903063319585255179509"
    );

    let mut selector = transcoder.encode("op_u256", ["shl"])?;
    U256::from_dec_str("10000000000000")?.encode_to(&mut selector);
    U256::from_dec_str("8")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, U256::from(2560000000000000_u128));

    let mut selector = transcoder.encode("op_u256", ["shr"])?;
    U256::from_dec_str("10000000000000")?.encode_to(&mut selector);
    U256::from_dec_str("8")?.encode_to(&mut selector);

    let res = query::<U256>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, U256::from(39062500000_u128));

    // test bytesN
    let selector = transcoder.encode::<_, String>("return_u8_6", [])?;

    let res = query::<[u8; 6]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "414243444546");

    // test bytesS
    let selector = transcoder.encode("op_u8_5_shift", ["shl", "0xdeadcafe59", "8"])?;

    let res = query::<[u8; 5]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "adcafe5900");

    let selector = transcoder.encode("op_u8_5_shift", ["shr", "0xdeadcafe59", "8"])?;

    let res = query::<[u8; 5]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "00deadcafe");

    // opU85
    let selector = transcoder.encode("op_u8_5", ["or", "0xdeadcafe59", "0x0000000006"])?;

    let res = query::<[u8; 5]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "deadcafe5f");

    let selector = transcoder.encode("op_u8_5", ["and", "0xdeadcafe59", "0x00000000ff"])?;

    let res = query::<[u8; 5]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "0000000059");

    let selector = transcoder.encode("op_u8_5", ["xor", "0xdeadcafe59", "0x00000000ff"])?;

    let res = query::<[u8; 5]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "deadcafea6");

    // test bytes14
    let selector = transcoder.encode(
        "op_u8_14_shift",
        ["shl", "0xdeadcafe123456789abcdefbeef7", "9"],
    )?;

    let res = query::<[u8; 14]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "5b95fc2468acf13579bdf7ddee00");

    let selector = transcoder.encode(
        "op_u8_14_shift",
        ["shr", "0xdeadcafe123456789abcdefbeef7", "9"],
    )?;
    let res = query::<[u8; 14]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "006f56e57f091a2b3c4d5e6f7df7");

    let selector = transcoder.encode(
        "op_u8_14",
        [
            "or",
            "0xdeadcafe123456789abcdefbeef7",
            "0x0000060000000000000000000000",
        ],
    )?;

    let res = query::<[u8; 14]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "deadcefe123456789abcdefbeef7");

    let selector = transcoder.encode(
        "op_u8_14",
        [
            "and",
            "0xdeadcafe123456789abcdefbeef7",
            "0x000000000000000000ff00000000",
        ],
    )?;

    let res = query::<[u8; 14]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "000000000000000000bc00000000");

    let selector = transcoder.encode(
        "op_u8_14",
        [
            "xor",
            "0xdeadcafe123456789abcdefbeef7",
            "0xff00000000000000000000000000",
        ],
    )?;

    let res = query::<[u8; 14]>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(hex::encode(res), "21adcafe123456789abcdefbeef7");

    // test addressPassthrough
    let default_acc =
        AccountId32::from_str("5GBWmgdFAMqm8ZgAHGobqDqX6tjLxJhv53ygjNtaaAn3sjeZ").unwrap();

    let selector = transcoder.encode(
        "address_passthrough",
        [format!("0x{}", hex::encode(&default_acc))],
    )?;

    let res = query::<AccountId32>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, default_acc);

    let alice = sp_keyring::AccountKeyring::Alice.to_account_id();

    let dave = sp_keyring::AccountKeyring::Dave.to_account_id();

    let selector = transcoder.encode(
        "address_passthrough",
        [format!("0x{}", hex::encode(&alice))],
    )?;

    let res = query::<AccountId32>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, alice);

    let selector =
        transcoder.encode("address_passthrough", [format!("0x{}", hex::encode(&dave))])?;

    let res = query::<AccountId32>(&api, &contract.contract_address, &selector).await?;

    assert_eq!(res, dave);

    Ok(())
}
