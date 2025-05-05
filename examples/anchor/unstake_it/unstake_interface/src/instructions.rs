use crate::*;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::io::Read;
#[derive(Clone, Debug, PartialEq)]
pub enum UnstakeProgramIx {
    InitProtocolFee,
    SetProtocolFee(SetProtocolFeeIxArgs),
    CreatePool(CreatePoolIxArgs),
    AddLiquidity(AddLiquidityIxArgs),
    RemoveLiquidity(RemoveLiquidityIxArgs),
    SetFee(SetFeeIxArgs),
    SetFeeAuthority,
    DeactivateStakeAccount,
    ReclaimStakeAccount,
    Unstake,
    UnstakeWsol,
}
impl UnstakeProgramIx {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        match maybe_discm {
            INIT_PROTOCOL_FEE_IX_DISCM => Ok(Self::InitProtocolFee),
            SET_PROTOCOL_FEE_IX_DISCM => Ok(Self::SetProtocolFee(
                SetProtocolFeeIxArgs::deserialize(&mut reader)?,
            )),
            CREATE_POOL_IX_DISCM => Ok(Self::CreatePool(CreatePoolIxArgs::deserialize(
                &mut reader,
            )?)),
            ADD_LIQUIDITY_IX_DISCM => Ok(Self::AddLiquidity(AddLiquidityIxArgs::deserialize(
                &mut reader,
            )?)),
            REMOVE_LIQUIDITY_IX_DISCM => Ok(Self::RemoveLiquidity(
                RemoveLiquidityIxArgs::deserialize(&mut reader)?,
            )),
            SET_FEE_IX_DISCM => Ok(Self::SetFee(SetFeeIxArgs::deserialize(&mut reader)?)),
            SET_FEE_AUTHORITY_IX_DISCM => Ok(Self::SetFeeAuthority),
            DEACTIVATE_STAKE_ACCOUNT_IX_DISCM => Ok(Self::DeactivateStakeAccount),
            RECLAIM_STAKE_ACCOUNT_IX_DISCM => Ok(Self::ReclaimStakeAccount),
            UNSTAKE_IX_DISCM => Ok(Self::Unstake),
            UNSTAKE_WSOL_IX_DISCM => Ok(Self::UnstakeWsol),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("discm {:?} not found", maybe_discm),
            )),
        }
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        match self {
            Self::InitProtocolFee => writer.write_all(&INIT_PROTOCOL_FEE_IX_DISCM),
            Self::SetProtocolFee(args) => {
                writer.write_all(&SET_PROTOCOL_FEE_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::CreatePool(args) => {
                writer.write_all(&CREATE_POOL_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::AddLiquidity(args) => {
                writer.write_all(&ADD_LIQUIDITY_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::RemoveLiquidity(args) => {
                writer.write_all(&REMOVE_LIQUIDITY_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::SetFee(args) => {
                writer.write_all(&SET_FEE_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::SetFeeAuthority => writer.write_all(&SET_FEE_AUTHORITY_IX_DISCM),
            Self::DeactivateStakeAccount => writer.write_all(&DEACTIVATE_STAKE_ACCOUNT_IX_DISCM),
            Self::ReclaimStakeAccount => writer.write_all(&RECLAIM_STAKE_ACCOUNT_IX_DISCM),
            Self::Unstake => writer.write_all(&UNSTAKE_IX_DISCM),
            Self::UnstakeWsol => writer.write_all(&UNSTAKE_WSOL_IX_DISCM),
        }
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
fn invoke_instruction<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
    ix: &Instruction,
    accounts: A,
) -> ProgramResult {
    let account_info: [AccountInfo<'info>; N] = accounts.into();
    invoke(ix, &account_info)
}
fn invoke_instruction_signed<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
    ix: &Instruction,
    accounts: A,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let account_info: [AccountInfo<'info>; N] = accounts.into();
    invoke_signed(ix, &account_info, seeds)
}
pub const INIT_PROTOCOL_FEE_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct InitProtocolFeeAccounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub protocol_fee_account: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InitProtocolFeeKeys {
    pub payer: Pubkey,
    pub protocol_fee_account: Pubkey,
    pub system_program: Pubkey,
}
impl From<InitProtocolFeeAccounts<'_, '_>> for InitProtocolFeeKeys {
    fn from(accounts: InitProtocolFeeAccounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            protocol_fee_account: *accounts.protocol_fee_account.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<InitProtocolFeeKeys> for [AccountMeta; INIT_PROTOCOL_FEE_IX_ACCOUNTS_LEN] {
    fn from(keys: InitProtocolFeeKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; INIT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]> for InitProtocolFeeKeys {
    fn from(pubkeys: [Pubkey; INIT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            protocol_fee_account: pubkeys[1],
            system_program: pubkeys[2],
        }
    }
}
impl<'info> From<InitProtocolFeeAccounts<'_, 'info>>
    for [AccountInfo<'info>; INIT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]
{
    fn from(accounts: InitProtocolFeeAccounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.protocol_fee_account.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; INIT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]>
    for InitProtocolFeeAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; INIT_PROTOCOL_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            protocol_fee_account: &arr[1],
            system_program: &arr[2],
        }
    }
}
pub const INIT_PROTOCOL_FEE_IX_DISCM: [u8; 8] = [225, 155, 167, 170, 29, 145, 165, 90];
#[derive(Clone, Debug, PartialEq)]
pub struct InitProtocolFeeIxData;
impl InitProtocolFeeIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != INIT_PROTOCOL_FEE_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    INIT_PROTOCOL_FEE_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&INIT_PROTOCOL_FEE_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn init_protocol_fee_ix_with_program_id(
    program_id: Pubkey,
    keys: InitProtocolFeeKeys,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; INIT_PROTOCOL_FEE_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: InitProtocolFeeIxData.try_to_vec()?,
    })
}
pub fn init_protocol_fee_ix(keys: InitProtocolFeeKeys) -> std::io::Result<Instruction> {
    init_protocol_fee_ix_with_program_id(crate::ID, keys)
}
pub fn init_protocol_fee_invoke_with_program_id(
    program_id: Pubkey,
    accounts: InitProtocolFeeAccounts<'_, '_>,
) -> ProgramResult {
    let keys: InitProtocolFeeKeys = accounts.into();
    let ix = init_protocol_fee_ix_with_program_id(program_id, keys)?;
    invoke_instruction(&ix, accounts)
}
pub fn init_protocol_fee_invoke(accounts: InitProtocolFeeAccounts<'_, '_>) -> ProgramResult {
    init_protocol_fee_invoke_with_program_id(crate::ID, accounts)
}
pub fn init_protocol_fee_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: InitProtocolFeeAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: InitProtocolFeeKeys = accounts.into();
    let ix = init_protocol_fee_ix_with_program_id(program_id, keys)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn init_protocol_fee_invoke_signed(
    accounts: InitProtocolFeeAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    init_protocol_fee_invoke_signed_with_program_id(crate::ID, accounts, seeds)
}
pub fn init_protocol_fee_verify_account_keys(
    accounts: InitProtocolFeeAccounts<'_, '_>,
    keys: InitProtocolFeeKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.payer.key, keys.payer),
        (
            *accounts.protocol_fee_account.key,
            keys.protocol_fee_account,
        ),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn init_protocol_fee_verify_writable_privileges<'me, 'info>(
    accounts: InitProtocolFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.payer, accounts.protocol_fee_account] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn init_protocol_fee_verify_signer_privileges<'me, 'info>(
    accounts: InitProtocolFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn init_protocol_fee_verify_account_privileges<'me, 'info>(
    accounts: InitProtocolFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    init_protocol_fee_verify_writable_privileges(accounts)?;
    init_protocol_fee_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const SET_PROTOCOL_FEE_IX_ACCOUNTS_LEN: usize = 2;
#[derive(Copy, Clone, Debug)]
pub struct SetProtocolFeeAccounts<'me, 'info> {
    pub authority: &'me AccountInfo<'info>,
    pub protocol_fee_account: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SetProtocolFeeKeys {
    pub authority: Pubkey,
    pub protocol_fee_account: Pubkey,
}
impl From<SetProtocolFeeAccounts<'_, '_>> for SetProtocolFeeKeys {
    fn from(accounts: SetProtocolFeeAccounts) -> Self {
        Self {
            authority: *accounts.authority.key,
            protocol_fee_account: *accounts.protocol_fee_account.key,
        }
    }
}
impl From<SetProtocolFeeKeys> for [AccountMeta; SET_PROTOCOL_FEE_IX_ACCOUNTS_LEN] {
    fn from(keys: SetProtocolFeeKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_account,
                is_signer: false,
                is_writable: true,
            },
        ]
    }
}
impl From<[Pubkey; SET_PROTOCOL_FEE_IX_ACCOUNTS_LEN]> for SetProtocolFeeKeys {
    fn from(pubkeys: [Pubkey; SET_PROTOCOL_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: pubkeys[0],
            protocol_fee_account: pubkeys[1],
        }
    }
}
impl<'info> From<SetProtocolFeeAccounts<'_, 'info>>
    for [AccountInfo<'info>; SET_PROTOCOL_FEE_IX_ACCOUNTS_LEN]
{
    fn from(accounts: SetProtocolFeeAccounts<'_, 'info>) -> Self {
        [
            accounts.authority.clone(),
            accounts.protocol_fee_account.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SET_PROTOCOL_FEE_IX_ACCOUNTS_LEN]>
    for SetProtocolFeeAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; SET_PROTOCOL_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            authority: &arr[0],
            protocol_fee_account: &arr[1],
        }
    }
}
pub const SET_PROTOCOL_FEE_IX_DISCM: [u8; 8] = [173, 239, 83, 242, 136, 43, 144, 217];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetProtocolFeeIxArgs {
    pub protocol_fee: ProtocolFee,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SetProtocolFeeIxData(pub SetProtocolFeeIxArgs);
impl From<SetProtocolFeeIxArgs> for SetProtocolFeeIxData {
    fn from(args: SetProtocolFeeIxArgs) -> Self {
        Self(args)
    }
}
impl SetProtocolFeeIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SET_PROTOCOL_FEE_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    SET_PROTOCOL_FEE_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self(SetProtocolFeeIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SET_PROTOCOL_FEE_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn set_protocol_fee_ix_with_program_id(
    program_id: Pubkey,
    keys: SetProtocolFeeKeys,
    args: SetProtocolFeeIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; SET_PROTOCOL_FEE_IX_ACCOUNTS_LEN] = keys.into();
    let data: SetProtocolFeeIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn set_protocol_fee_ix(
    keys: SetProtocolFeeKeys,
    args: SetProtocolFeeIxArgs,
) -> std::io::Result<Instruction> {
    set_protocol_fee_ix_with_program_id(crate::ID, keys, args)
}
pub fn set_protocol_fee_invoke_with_program_id(
    program_id: Pubkey,
    accounts: SetProtocolFeeAccounts<'_, '_>,
    args: SetProtocolFeeIxArgs,
) -> ProgramResult {
    let keys: SetProtocolFeeKeys = accounts.into();
    let ix = set_protocol_fee_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn set_protocol_fee_invoke(
    accounts: SetProtocolFeeAccounts<'_, '_>,
    args: SetProtocolFeeIxArgs,
) -> ProgramResult {
    set_protocol_fee_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn set_protocol_fee_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: SetProtocolFeeAccounts<'_, '_>,
    args: SetProtocolFeeIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: SetProtocolFeeKeys = accounts.into();
    let ix = set_protocol_fee_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn set_protocol_fee_invoke_signed(
    accounts: SetProtocolFeeAccounts<'_, '_>,
    args: SetProtocolFeeIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    set_protocol_fee_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn set_protocol_fee_verify_account_keys(
    accounts: SetProtocolFeeAccounts<'_, '_>,
    keys: SetProtocolFeeKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.authority.key, keys.authority),
        (
            *accounts.protocol_fee_account.key,
            keys.protocol_fee_account,
        ),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn set_protocol_fee_verify_writable_privileges<'me, 'info>(
    accounts: SetProtocolFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.protocol_fee_account] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn set_protocol_fee_verify_signer_privileges<'me, 'info>(
    accounts: SetProtocolFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn set_protocol_fee_verify_account_privileges<'me, 'info>(
    accounts: SetProtocolFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    set_protocol_fee_verify_writable_privileges(accounts)?;
    set_protocol_fee_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const CREATE_POOL_IX_ACCOUNTS_LEN: usize = 9;
#[derive(Copy, Clone, Debug)]
pub struct CreatePoolAccounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub fee_authority: &'me AccountInfo<'info>,
    pub pool_account: &'me AccountInfo<'info>,
    pub pool_sol_reserves: &'me AccountInfo<'info>,
    pub fee_account: &'me AccountInfo<'info>,
    pub lp_mint: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CreatePoolKeys {
    pub payer: Pubkey,
    pub fee_authority: Pubkey,
    pub pool_account: Pubkey,
    pub pool_sol_reserves: Pubkey,
    pub fee_account: Pubkey,
    pub lp_mint: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<CreatePoolAccounts<'_, '_>> for CreatePoolKeys {
    fn from(accounts: CreatePoolAccounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            fee_authority: *accounts.fee_authority.key,
            pool_account: *accounts.pool_account.key,
            pool_sol_reserves: *accounts.pool_sol_reserves.key,
            fee_account: *accounts.fee_account.key,
            lp_mint: *accounts.lp_mint.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<CreatePoolKeys> for [AccountMeta; CREATE_POOL_IX_ACCOUNTS_LEN] {
    fn from(keys: CreatePoolKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.fee_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_account,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_sol_reserves,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.fee_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.lp_mint,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.rent,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; CREATE_POOL_IX_ACCOUNTS_LEN]> for CreatePoolKeys {
    fn from(pubkeys: [Pubkey; CREATE_POOL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            fee_authority: pubkeys[1],
            pool_account: pubkeys[2],
            pool_sol_reserves: pubkeys[3],
            fee_account: pubkeys[4],
            lp_mint: pubkeys[5],
            token_program: pubkeys[6],
            system_program: pubkeys[7],
            rent: pubkeys[8],
        }
    }
}
impl<'info> From<CreatePoolAccounts<'_, 'info>>
    for [AccountInfo<'info>; CREATE_POOL_IX_ACCOUNTS_LEN]
{
    fn from(accounts: CreatePoolAccounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.fee_authority.clone(),
            accounts.pool_account.clone(),
            accounts.pool_sol_reserves.clone(),
            accounts.fee_account.clone(),
            accounts.lp_mint.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CREATE_POOL_IX_ACCOUNTS_LEN]>
    for CreatePoolAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; CREATE_POOL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            fee_authority: &arr[1],
            pool_account: &arr[2],
            pool_sol_reserves: &arr[3],
            fee_account: &arr[4],
            lp_mint: &arr[5],
            token_program: &arr[6],
            system_program: &arr[7],
            rent: &arr[8],
        }
    }
}
pub const CREATE_POOL_IX_DISCM: [u8; 8] = [233, 146, 209, 142, 207, 104, 64, 188];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreatePoolIxArgs {
    pub fee: Fee,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CreatePoolIxData(pub CreatePoolIxArgs);
impl From<CreatePoolIxArgs> for CreatePoolIxData {
    fn from(args: CreatePoolIxArgs) -> Self {
        Self(args)
    }
}
impl CreatePoolIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CREATE_POOL_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    CREATE_POOL_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self(CreatePoolIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CREATE_POOL_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn create_pool_ix_with_program_id(
    program_id: Pubkey,
    keys: CreatePoolKeys,
    args: CreatePoolIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CREATE_POOL_IX_ACCOUNTS_LEN] = keys.into();
    let data: CreatePoolIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn create_pool_ix(
    keys: CreatePoolKeys,
    args: CreatePoolIxArgs,
) -> std::io::Result<Instruction> {
    create_pool_ix_with_program_id(crate::ID, keys, args)
}
pub fn create_pool_invoke_with_program_id(
    program_id: Pubkey,
    accounts: CreatePoolAccounts<'_, '_>,
    args: CreatePoolIxArgs,
) -> ProgramResult {
    let keys: CreatePoolKeys = accounts.into();
    let ix = create_pool_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn create_pool_invoke(
    accounts: CreatePoolAccounts<'_, '_>,
    args: CreatePoolIxArgs,
) -> ProgramResult {
    create_pool_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn create_pool_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: CreatePoolAccounts<'_, '_>,
    args: CreatePoolIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: CreatePoolKeys = accounts.into();
    let ix = create_pool_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn create_pool_invoke_signed(
    accounts: CreatePoolAccounts<'_, '_>,
    args: CreatePoolIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    create_pool_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn create_pool_verify_account_keys(
    accounts: CreatePoolAccounts<'_, '_>,
    keys: CreatePoolKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.payer.key, keys.payer),
        (*accounts.fee_authority.key, keys.fee_authority),
        (*accounts.pool_account.key, keys.pool_account),
        (*accounts.pool_sol_reserves.key, keys.pool_sol_reserves),
        (*accounts.fee_account.key, keys.fee_account),
        (*accounts.lp_mint.key, keys.lp_mint),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.system_program.key, keys.system_program),
        (*accounts.rent.key, keys.rent),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn create_pool_verify_writable_privileges<'me, 'info>(
    accounts: CreatePoolAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.payer,
        accounts.pool_account,
        accounts.fee_account,
        accounts.lp_mint,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn create_pool_verify_signer_privileges<'me, 'info>(
    accounts: CreatePoolAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [
        accounts.payer,
        accounts.fee_authority,
        accounts.pool_account,
        accounts.lp_mint,
    ] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn create_pool_verify_account_privileges<'me, 'info>(
    accounts: CreatePoolAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    create_pool_verify_writable_privileges(accounts)?;
    create_pool_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const ADD_LIQUIDITY_IX_ACCOUNTS_LEN: usize = 7;
#[derive(Copy, Clone, Debug)]
pub struct AddLiquidityAccounts<'me, 'info> {
    pub from: &'me AccountInfo<'info>,
    pub pool_account: &'me AccountInfo<'info>,
    pub pool_sol_reserves: &'me AccountInfo<'info>,
    pub lp_mint: &'me AccountInfo<'info>,
    pub mint_lp_tokens_to: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AddLiquidityKeys {
    pub from: Pubkey,
    pub pool_account: Pubkey,
    pub pool_sol_reserves: Pubkey,
    pub lp_mint: Pubkey,
    pub mint_lp_tokens_to: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
}
impl From<AddLiquidityAccounts<'_, '_>> for AddLiquidityKeys {
    fn from(accounts: AddLiquidityAccounts) -> Self {
        Self {
            from: *accounts.from.key,
            pool_account: *accounts.pool_account.key,
            pool_sol_reserves: *accounts.pool_sol_reserves.key,
            lp_mint: *accounts.lp_mint.key,
            mint_lp_tokens_to: *accounts.mint_lp_tokens_to.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<AddLiquidityKeys> for [AccountMeta; ADD_LIQUIDITY_IX_ACCOUNTS_LEN] {
    fn from(keys: AddLiquidityKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.from,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_sol_reserves,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.lp_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.mint_lp_tokens_to,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; ADD_LIQUIDITY_IX_ACCOUNTS_LEN]> for AddLiquidityKeys {
    fn from(pubkeys: [Pubkey; ADD_LIQUIDITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            from: pubkeys[0],
            pool_account: pubkeys[1],
            pool_sol_reserves: pubkeys[2],
            lp_mint: pubkeys[3],
            mint_lp_tokens_to: pubkeys[4],
            token_program: pubkeys[5],
            system_program: pubkeys[6],
        }
    }
}
impl<'info> From<AddLiquidityAccounts<'_, 'info>>
    for [AccountInfo<'info>; ADD_LIQUIDITY_IX_ACCOUNTS_LEN]
{
    fn from(accounts: AddLiquidityAccounts<'_, 'info>) -> Self {
        [
            accounts.from.clone(),
            accounts.pool_account.clone(),
            accounts.pool_sol_reserves.clone(),
            accounts.lp_mint.clone(),
            accounts.mint_lp_tokens_to.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; ADD_LIQUIDITY_IX_ACCOUNTS_LEN]>
    for AddLiquidityAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; ADD_LIQUIDITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            from: &arr[0],
            pool_account: &arr[1],
            pool_sol_reserves: &arr[2],
            lp_mint: &arr[3],
            mint_lp_tokens_to: &arr[4],
            token_program: &arr[5],
            system_program: &arr[6],
        }
    }
}
pub const ADD_LIQUIDITY_IX_DISCM: [u8; 8] = [181, 157, 89, 67, 143, 182, 52, 72];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AddLiquidityIxArgs {
    pub amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct AddLiquidityIxData(pub AddLiquidityIxArgs);
impl From<AddLiquidityIxArgs> for AddLiquidityIxData {
    fn from(args: AddLiquidityIxArgs) -> Self {
        Self(args)
    }
}
impl AddLiquidityIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != ADD_LIQUIDITY_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    ADD_LIQUIDITY_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self(AddLiquidityIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&ADD_LIQUIDITY_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn add_liquidity_ix_with_program_id(
    program_id: Pubkey,
    keys: AddLiquidityKeys,
    args: AddLiquidityIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; ADD_LIQUIDITY_IX_ACCOUNTS_LEN] = keys.into();
    let data: AddLiquidityIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn add_liquidity_ix(
    keys: AddLiquidityKeys,
    args: AddLiquidityIxArgs,
) -> std::io::Result<Instruction> {
    add_liquidity_ix_with_program_id(crate::ID, keys, args)
}
pub fn add_liquidity_invoke_with_program_id(
    program_id: Pubkey,
    accounts: AddLiquidityAccounts<'_, '_>,
    args: AddLiquidityIxArgs,
) -> ProgramResult {
    let keys: AddLiquidityKeys = accounts.into();
    let ix = add_liquidity_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn add_liquidity_invoke(
    accounts: AddLiquidityAccounts<'_, '_>,
    args: AddLiquidityIxArgs,
) -> ProgramResult {
    add_liquidity_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn add_liquidity_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: AddLiquidityAccounts<'_, '_>,
    args: AddLiquidityIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: AddLiquidityKeys = accounts.into();
    let ix = add_liquidity_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn add_liquidity_invoke_signed(
    accounts: AddLiquidityAccounts<'_, '_>,
    args: AddLiquidityIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    add_liquidity_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn add_liquidity_verify_account_keys(
    accounts: AddLiquidityAccounts<'_, '_>,
    keys: AddLiquidityKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.from.key, keys.from),
        (*accounts.pool_account.key, keys.pool_account),
        (*accounts.pool_sol_reserves.key, keys.pool_sol_reserves),
        (*accounts.lp_mint.key, keys.lp_mint),
        (*accounts.mint_lp_tokens_to.key, keys.mint_lp_tokens_to),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn add_liquidity_verify_writable_privileges<'me, 'info>(
    accounts: AddLiquidityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.from,
        accounts.pool_account,
        accounts.pool_sol_reserves,
        accounts.lp_mint,
        accounts.mint_lp_tokens_to,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn add_liquidity_verify_signer_privileges<'me, 'info>(
    accounts: AddLiquidityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.from] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn add_liquidity_verify_account_privileges<'me, 'info>(
    accounts: AddLiquidityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    add_liquidity_verify_writable_privileges(accounts)?;
    add_liquidity_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const REMOVE_LIQUIDITY_IX_ACCOUNTS_LEN: usize = 8;
#[derive(Copy, Clone, Debug)]
pub struct RemoveLiquidityAccounts<'me, 'info> {
    pub burn_lp_tokens_from_authority: &'me AccountInfo<'info>,
    pub to: &'me AccountInfo<'info>,
    pub pool_account: &'me AccountInfo<'info>,
    pub pool_sol_reserves: &'me AccountInfo<'info>,
    pub lp_mint: &'me AccountInfo<'info>,
    pub burn_lp_tokens_from: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RemoveLiquidityKeys {
    pub burn_lp_tokens_from_authority: Pubkey,
    pub to: Pubkey,
    pub pool_account: Pubkey,
    pub pool_sol_reserves: Pubkey,
    pub lp_mint: Pubkey,
    pub burn_lp_tokens_from: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
}
impl From<RemoveLiquidityAccounts<'_, '_>> for RemoveLiquidityKeys {
    fn from(accounts: RemoveLiquidityAccounts) -> Self {
        Self {
            burn_lp_tokens_from_authority: *accounts.burn_lp_tokens_from_authority.key,
            to: *accounts.to.key,
            pool_account: *accounts.pool_account.key,
            pool_sol_reserves: *accounts.pool_sol_reserves.key,
            lp_mint: *accounts.lp_mint.key,
            burn_lp_tokens_from: *accounts.burn_lp_tokens_from.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<RemoveLiquidityKeys> for [AccountMeta; REMOVE_LIQUIDITY_IX_ACCOUNTS_LEN] {
    fn from(keys: RemoveLiquidityKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.burn_lp_tokens_from_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.to,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_sol_reserves,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.lp_mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.burn_lp_tokens_from,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; REMOVE_LIQUIDITY_IX_ACCOUNTS_LEN]> for RemoveLiquidityKeys {
    fn from(pubkeys: [Pubkey; REMOVE_LIQUIDITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            burn_lp_tokens_from_authority: pubkeys[0],
            to: pubkeys[1],
            pool_account: pubkeys[2],
            pool_sol_reserves: pubkeys[3],
            lp_mint: pubkeys[4],
            burn_lp_tokens_from: pubkeys[5],
            token_program: pubkeys[6],
            system_program: pubkeys[7],
        }
    }
}
impl<'info> From<RemoveLiquidityAccounts<'_, 'info>>
    for [AccountInfo<'info>; REMOVE_LIQUIDITY_IX_ACCOUNTS_LEN]
{
    fn from(accounts: RemoveLiquidityAccounts<'_, 'info>) -> Self {
        [
            accounts.burn_lp_tokens_from_authority.clone(),
            accounts.to.clone(),
            accounts.pool_account.clone(),
            accounts.pool_sol_reserves.clone(),
            accounts.lp_mint.clone(),
            accounts.burn_lp_tokens_from.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; REMOVE_LIQUIDITY_IX_ACCOUNTS_LEN]>
    for RemoveLiquidityAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; REMOVE_LIQUIDITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            burn_lp_tokens_from_authority: &arr[0],
            to: &arr[1],
            pool_account: &arr[2],
            pool_sol_reserves: &arr[3],
            lp_mint: &arr[4],
            burn_lp_tokens_from: &arr[5],
            token_program: &arr[6],
            system_program: &arr[7],
        }
    }
}
pub const REMOVE_LIQUIDITY_IX_DISCM: [u8; 8] = [80, 85, 209, 72, 24, 206, 177, 108];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RemoveLiquidityIxArgs {
    pub amount_lp: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RemoveLiquidityIxData(pub RemoveLiquidityIxArgs);
impl From<RemoveLiquidityIxArgs> for RemoveLiquidityIxData {
    fn from(args: RemoveLiquidityIxArgs) -> Self {
        Self(args)
    }
}
impl RemoveLiquidityIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != REMOVE_LIQUIDITY_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    REMOVE_LIQUIDITY_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self(RemoveLiquidityIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&REMOVE_LIQUIDITY_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn remove_liquidity_ix_with_program_id(
    program_id: Pubkey,
    keys: RemoveLiquidityKeys,
    args: RemoveLiquidityIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; REMOVE_LIQUIDITY_IX_ACCOUNTS_LEN] = keys.into();
    let data: RemoveLiquidityIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn remove_liquidity_ix(
    keys: RemoveLiquidityKeys,
    args: RemoveLiquidityIxArgs,
) -> std::io::Result<Instruction> {
    remove_liquidity_ix_with_program_id(crate::ID, keys, args)
}
pub fn remove_liquidity_invoke_with_program_id(
    program_id: Pubkey,
    accounts: RemoveLiquidityAccounts<'_, '_>,
    args: RemoveLiquidityIxArgs,
) -> ProgramResult {
    let keys: RemoveLiquidityKeys = accounts.into();
    let ix = remove_liquidity_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn remove_liquidity_invoke(
    accounts: RemoveLiquidityAccounts<'_, '_>,
    args: RemoveLiquidityIxArgs,
) -> ProgramResult {
    remove_liquidity_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn remove_liquidity_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: RemoveLiquidityAccounts<'_, '_>,
    args: RemoveLiquidityIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: RemoveLiquidityKeys = accounts.into();
    let ix = remove_liquidity_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn remove_liquidity_invoke_signed(
    accounts: RemoveLiquidityAccounts<'_, '_>,
    args: RemoveLiquidityIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    remove_liquidity_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn remove_liquidity_verify_account_keys(
    accounts: RemoveLiquidityAccounts<'_, '_>,
    keys: RemoveLiquidityKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (
            *accounts.burn_lp_tokens_from_authority.key,
            keys.burn_lp_tokens_from_authority,
        ),
        (*accounts.to.key, keys.to),
        (*accounts.pool_account.key, keys.pool_account),
        (*accounts.pool_sol_reserves.key, keys.pool_sol_reserves),
        (*accounts.lp_mint.key, keys.lp_mint),
        (*accounts.burn_lp_tokens_from.key, keys.burn_lp_tokens_from),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn remove_liquidity_verify_writable_privileges<'me, 'info>(
    accounts: RemoveLiquidityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.to,
        accounts.pool_account,
        accounts.pool_sol_reserves,
        accounts.lp_mint,
        accounts.burn_lp_tokens_from,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn remove_liquidity_verify_signer_privileges<'me, 'info>(
    accounts: RemoveLiquidityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.burn_lp_tokens_from_authority] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn remove_liquidity_verify_account_privileges<'me, 'info>(
    accounts: RemoveLiquidityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    remove_liquidity_verify_writable_privileges(accounts)?;
    remove_liquidity_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const SET_FEE_IX_ACCOUNTS_LEN: usize = 5;
#[derive(Copy, Clone, Debug)]
pub struct SetFeeAccounts<'me, 'info> {
    pub fee_authority: &'me AccountInfo<'info>,
    pub pool_account: &'me AccountInfo<'info>,
    pub fee_account: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SetFeeKeys {
    pub fee_authority: Pubkey,
    pub pool_account: Pubkey,
    pub fee_account: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<SetFeeAccounts<'_, '_>> for SetFeeKeys {
    fn from(accounts: SetFeeAccounts) -> Self {
        Self {
            fee_authority: *accounts.fee_authority.key,
            pool_account: *accounts.pool_account.key,
            fee_account: *accounts.fee_account.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<SetFeeKeys> for [AccountMeta; SET_FEE_IX_ACCOUNTS_LEN] {
    fn from(keys: SetFeeKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.fee_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.fee_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.rent,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; SET_FEE_IX_ACCOUNTS_LEN]> for SetFeeKeys {
    fn from(pubkeys: [Pubkey; SET_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            fee_authority: pubkeys[0],
            pool_account: pubkeys[1],
            fee_account: pubkeys[2],
            system_program: pubkeys[3],
            rent: pubkeys[4],
        }
    }
}
impl<'info> From<SetFeeAccounts<'_, 'info>> for [AccountInfo<'info>; SET_FEE_IX_ACCOUNTS_LEN] {
    fn from(accounts: SetFeeAccounts<'_, 'info>) -> Self {
        [
            accounts.fee_authority.clone(),
            accounts.pool_account.clone(),
            accounts.fee_account.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SET_FEE_IX_ACCOUNTS_LEN]>
    for SetFeeAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; SET_FEE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            fee_authority: &arr[0],
            pool_account: &arr[1],
            fee_account: &arr[2],
            system_program: &arr[3],
            rent: &arr[4],
        }
    }
}
pub const SET_FEE_IX_DISCM: [u8; 8] = [18, 154, 24, 18, 237, 214, 19, 80];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetFeeIxArgs {
    pub fee: Fee,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SetFeeIxData(pub SetFeeIxArgs);
impl From<SetFeeIxArgs> for SetFeeIxData {
    fn from(args: SetFeeIxArgs) -> Self {
        Self(args)
    }
}
impl SetFeeIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SET_FEE_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    SET_FEE_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self(SetFeeIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SET_FEE_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn set_fee_ix_with_program_id(
    program_id: Pubkey,
    keys: SetFeeKeys,
    args: SetFeeIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; SET_FEE_IX_ACCOUNTS_LEN] = keys.into();
    let data: SetFeeIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn set_fee_ix(keys: SetFeeKeys, args: SetFeeIxArgs) -> std::io::Result<Instruction> {
    set_fee_ix_with_program_id(crate::ID, keys, args)
}
pub fn set_fee_invoke_with_program_id(
    program_id: Pubkey,
    accounts: SetFeeAccounts<'_, '_>,
    args: SetFeeIxArgs,
) -> ProgramResult {
    let keys: SetFeeKeys = accounts.into();
    let ix = set_fee_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn set_fee_invoke(accounts: SetFeeAccounts<'_, '_>, args: SetFeeIxArgs) -> ProgramResult {
    set_fee_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn set_fee_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: SetFeeAccounts<'_, '_>,
    args: SetFeeIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: SetFeeKeys = accounts.into();
    let ix = set_fee_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn set_fee_invoke_signed(
    accounts: SetFeeAccounts<'_, '_>,
    args: SetFeeIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    set_fee_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn set_fee_verify_account_keys(
    accounts: SetFeeAccounts<'_, '_>,
    keys: SetFeeKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.fee_authority.key, keys.fee_authority),
        (*accounts.pool_account.key, keys.pool_account),
        (*accounts.fee_account.key, keys.fee_account),
        (*accounts.system_program.key, keys.system_program),
        (*accounts.rent.key, keys.rent),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn set_fee_verify_writable_privileges<'me, 'info>(
    accounts: SetFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.fee_account] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn set_fee_verify_signer_privileges<'me, 'info>(
    accounts: SetFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.fee_authority] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn set_fee_verify_account_privileges<'me, 'info>(
    accounts: SetFeeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    set_fee_verify_writable_privileges(accounts)?;
    set_fee_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const SET_FEE_AUTHORITY_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct SetFeeAuthorityAccounts<'me, 'info> {
    pub fee_authority: &'me AccountInfo<'info>,
    pub pool_account: &'me AccountInfo<'info>,
    pub new_fee_authority: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SetFeeAuthorityKeys {
    pub fee_authority: Pubkey,
    pub pool_account: Pubkey,
    pub new_fee_authority: Pubkey,
}
impl From<SetFeeAuthorityAccounts<'_, '_>> for SetFeeAuthorityKeys {
    fn from(accounts: SetFeeAuthorityAccounts) -> Self {
        Self {
            fee_authority: *accounts.fee_authority.key,
            pool_account: *accounts.pool_account.key,
            new_fee_authority: *accounts.new_fee_authority.key,
        }
    }
}
impl From<SetFeeAuthorityKeys> for [AccountMeta; SET_FEE_AUTHORITY_IX_ACCOUNTS_LEN] {
    fn from(keys: SetFeeAuthorityKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.fee_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.new_fee_authority,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; SET_FEE_AUTHORITY_IX_ACCOUNTS_LEN]> for SetFeeAuthorityKeys {
    fn from(pubkeys: [Pubkey; SET_FEE_AUTHORITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            fee_authority: pubkeys[0],
            pool_account: pubkeys[1],
            new_fee_authority: pubkeys[2],
        }
    }
}
impl<'info> From<SetFeeAuthorityAccounts<'_, 'info>>
    for [AccountInfo<'info>; SET_FEE_AUTHORITY_IX_ACCOUNTS_LEN]
{
    fn from(accounts: SetFeeAuthorityAccounts<'_, 'info>) -> Self {
        [
            accounts.fee_authority.clone(),
            accounts.pool_account.clone(),
            accounts.new_fee_authority.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SET_FEE_AUTHORITY_IX_ACCOUNTS_LEN]>
    for SetFeeAuthorityAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; SET_FEE_AUTHORITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            fee_authority: &arr[0],
            pool_account: &arr[1],
            new_fee_authority: &arr[2],
        }
    }
}
pub const SET_FEE_AUTHORITY_IX_DISCM: [u8; 8] = [31, 1, 50, 87, 237, 101, 97, 132];
#[derive(Clone, Debug, PartialEq)]
pub struct SetFeeAuthorityIxData;
impl SetFeeAuthorityIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SET_FEE_AUTHORITY_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    SET_FEE_AUTHORITY_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SET_FEE_AUTHORITY_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn set_fee_authority_ix_with_program_id(
    program_id: Pubkey,
    keys: SetFeeAuthorityKeys,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; SET_FEE_AUTHORITY_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: SetFeeAuthorityIxData.try_to_vec()?,
    })
}
pub fn set_fee_authority_ix(keys: SetFeeAuthorityKeys) -> std::io::Result<Instruction> {
    set_fee_authority_ix_with_program_id(crate::ID, keys)
}
pub fn set_fee_authority_invoke_with_program_id(
    program_id: Pubkey,
    accounts: SetFeeAuthorityAccounts<'_, '_>,
) -> ProgramResult {
    let keys: SetFeeAuthorityKeys = accounts.into();
    let ix = set_fee_authority_ix_with_program_id(program_id, keys)?;
    invoke_instruction(&ix, accounts)
}
pub fn set_fee_authority_invoke(accounts: SetFeeAuthorityAccounts<'_, '_>) -> ProgramResult {
    set_fee_authority_invoke_with_program_id(crate::ID, accounts)
}
pub fn set_fee_authority_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: SetFeeAuthorityAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: SetFeeAuthorityKeys = accounts.into();
    let ix = set_fee_authority_ix_with_program_id(program_id, keys)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn set_fee_authority_invoke_signed(
    accounts: SetFeeAuthorityAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    set_fee_authority_invoke_signed_with_program_id(crate::ID, accounts, seeds)
}
pub fn set_fee_authority_verify_account_keys(
    accounts: SetFeeAuthorityAccounts<'_, '_>,
    keys: SetFeeAuthorityKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.fee_authority.key, keys.fee_authority),
        (*accounts.pool_account.key, keys.pool_account),
        (*accounts.new_fee_authority.key, keys.new_fee_authority),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn set_fee_authority_verify_writable_privileges<'me, 'info>(
    accounts: SetFeeAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.pool_account] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn set_fee_authority_verify_signer_privileges<'me, 'info>(
    accounts: SetFeeAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.fee_authority] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn set_fee_authority_verify_account_privileges<'me, 'info>(
    accounts: SetFeeAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    set_fee_authority_verify_writable_privileges(accounts)?;
    set_fee_authority_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const DEACTIVATE_STAKE_ACCOUNT_IX_ACCOUNTS_LEN: usize = 5;
#[derive(Copy, Clone, Debug)]
pub struct DeactivateStakeAccountAccounts<'me, 'info> {
    pub stake_account: &'me AccountInfo<'info>,
    pub pool_account: &'me AccountInfo<'info>,
    pub pool_sol_reserves: &'me AccountInfo<'info>,
    pub clock: &'me AccountInfo<'info>,
    pub stake_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DeactivateStakeAccountKeys {
    pub stake_account: Pubkey,
    pub pool_account: Pubkey,
    pub pool_sol_reserves: Pubkey,
    pub clock: Pubkey,
    pub stake_program: Pubkey,
}
impl From<DeactivateStakeAccountAccounts<'_, '_>> for DeactivateStakeAccountKeys {
    fn from(accounts: DeactivateStakeAccountAccounts) -> Self {
        Self {
            stake_account: *accounts.stake_account.key,
            pool_account: *accounts.pool_account.key,
            pool_sol_reserves: *accounts.pool_sol_reserves.key,
            clock: *accounts.clock.key,
            stake_program: *accounts.stake_program.key,
        }
    }
}
impl From<DeactivateStakeAccountKeys> for [AccountMeta; DEACTIVATE_STAKE_ACCOUNT_IX_ACCOUNTS_LEN] {
    fn from(keys: DeactivateStakeAccountKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.stake_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.pool_sol_reserves,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.clock,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.stake_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; DEACTIVATE_STAKE_ACCOUNT_IX_ACCOUNTS_LEN]> for DeactivateStakeAccountKeys {
    fn from(pubkeys: [Pubkey; DEACTIVATE_STAKE_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            stake_account: pubkeys[0],
            pool_account: pubkeys[1],
            pool_sol_reserves: pubkeys[2],
            clock: pubkeys[3],
            stake_program: pubkeys[4],
        }
    }
}
impl<'info> From<DeactivateStakeAccountAccounts<'_, 'info>>
    for [AccountInfo<'info>; DEACTIVATE_STAKE_ACCOUNT_IX_ACCOUNTS_LEN]
{
    fn from(accounts: DeactivateStakeAccountAccounts<'_, 'info>) -> Self {
        [
            accounts.stake_account.clone(),
            accounts.pool_account.clone(),
            accounts.pool_sol_reserves.clone(),
            accounts.clock.clone(),
            accounts.stake_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; DEACTIVATE_STAKE_ACCOUNT_IX_ACCOUNTS_LEN]>
    for DeactivateStakeAccountAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; DEACTIVATE_STAKE_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            stake_account: &arr[0],
            pool_account: &arr[1],
            pool_sol_reserves: &arr[2],
            clock: &arr[3],
            stake_program: &arr[4],
        }
    }
}
pub const DEACTIVATE_STAKE_ACCOUNT_IX_DISCM: [u8; 8] = [217, 64, 76, 16, 216, 77, 123, 226];
#[derive(Clone, Debug, PartialEq)]
pub struct DeactivateStakeAccountIxData;
impl DeactivateStakeAccountIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != DEACTIVATE_STAKE_ACCOUNT_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    DEACTIVATE_STAKE_ACCOUNT_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&DEACTIVATE_STAKE_ACCOUNT_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn deactivate_stake_account_ix_with_program_id(
    program_id: Pubkey,
    keys: DeactivateStakeAccountKeys,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; DEACTIVATE_STAKE_ACCOUNT_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: DeactivateStakeAccountIxData.try_to_vec()?,
    })
}
pub fn deactivate_stake_account_ix(
    keys: DeactivateStakeAccountKeys,
) -> std::io::Result<Instruction> {
    deactivate_stake_account_ix_with_program_id(crate::ID, keys)
}
pub fn deactivate_stake_account_invoke_with_program_id(
    program_id: Pubkey,
    accounts: DeactivateStakeAccountAccounts<'_, '_>,
) -> ProgramResult {
    let keys: DeactivateStakeAccountKeys = accounts.into();
    let ix = deactivate_stake_account_ix_with_program_id(program_id, keys)?;
    invoke_instruction(&ix, accounts)
}
pub fn deactivate_stake_account_invoke(
    accounts: DeactivateStakeAccountAccounts<'_, '_>,
) -> ProgramResult {
    deactivate_stake_account_invoke_with_program_id(crate::ID, accounts)
}
pub fn deactivate_stake_account_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: DeactivateStakeAccountAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: DeactivateStakeAccountKeys = accounts.into();
    let ix = deactivate_stake_account_ix_with_program_id(program_id, keys)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn deactivate_stake_account_invoke_signed(
    accounts: DeactivateStakeAccountAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    deactivate_stake_account_invoke_signed_with_program_id(crate::ID, accounts, seeds)
}
pub fn deactivate_stake_account_verify_account_keys(
    accounts: DeactivateStakeAccountAccounts<'_, '_>,
    keys: DeactivateStakeAccountKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.stake_account.key, keys.stake_account),
        (*accounts.pool_account.key, keys.pool_account),
        (*accounts.pool_sol_reserves.key, keys.pool_sol_reserves),
        (*accounts.clock.key, keys.clock),
        (*accounts.stake_program.key, keys.stake_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn deactivate_stake_account_verify_writable_privileges<'me, 'info>(
    accounts: DeactivateStakeAccountAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.stake_account] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn deactivate_stake_account_verify_account_privileges<'me, 'info>(
    accounts: DeactivateStakeAccountAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    deactivate_stake_account_verify_writable_privileges(accounts)?;
    Ok(())
}
pub const RECLAIM_STAKE_ACCOUNT_IX_ACCOUNTS_LEN: usize = 7;
#[derive(Copy, Clone, Debug)]
pub struct ReclaimStakeAccountAccounts<'me, 'info> {
    pub stake_account: &'me AccountInfo<'info>,
    pub pool_account: &'me AccountInfo<'info>,
    pub pool_sol_reserves: &'me AccountInfo<'info>,
    pub stake_account_record_account: &'me AccountInfo<'info>,
    pub clock: &'me AccountInfo<'info>,
    pub stake_history: &'me AccountInfo<'info>,
    pub stake_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ReclaimStakeAccountKeys {
    pub stake_account: Pubkey,
    pub pool_account: Pubkey,
    pub pool_sol_reserves: Pubkey,
    pub stake_account_record_account: Pubkey,
    pub clock: Pubkey,
    pub stake_history: Pubkey,
    pub stake_program: Pubkey,
}
impl From<ReclaimStakeAccountAccounts<'_, '_>> for ReclaimStakeAccountKeys {
    fn from(accounts: ReclaimStakeAccountAccounts) -> Self {
        Self {
            stake_account: *accounts.stake_account.key,
            pool_account: *accounts.pool_account.key,
            pool_sol_reserves: *accounts.pool_sol_reserves.key,
            stake_account_record_account: *accounts.stake_account_record_account.key,
            clock: *accounts.clock.key,
            stake_history: *accounts.stake_history.key,
            stake_program: *accounts.stake_program.key,
        }
    }
}
impl From<ReclaimStakeAccountKeys> for [AccountMeta; RECLAIM_STAKE_ACCOUNT_IX_ACCOUNTS_LEN] {
    fn from(keys: ReclaimStakeAccountKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.stake_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_sol_reserves,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.stake_account_record_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.clock,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.stake_history,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.stake_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; RECLAIM_STAKE_ACCOUNT_IX_ACCOUNTS_LEN]> for ReclaimStakeAccountKeys {
    fn from(pubkeys: [Pubkey; RECLAIM_STAKE_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            stake_account: pubkeys[0],
            pool_account: pubkeys[1],
            pool_sol_reserves: pubkeys[2],
            stake_account_record_account: pubkeys[3],
            clock: pubkeys[4],
            stake_history: pubkeys[5],
            stake_program: pubkeys[6],
        }
    }
}
impl<'info> From<ReclaimStakeAccountAccounts<'_, 'info>>
    for [AccountInfo<'info>; RECLAIM_STAKE_ACCOUNT_IX_ACCOUNTS_LEN]
{
    fn from(accounts: ReclaimStakeAccountAccounts<'_, 'info>) -> Self {
        [
            accounts.stake_account.clone(),
            accounts.pool_account.clone(),
            accounts.pool_sol_reserves.clone(),
            accounts.stake_account_record_account.clone(),
            accounts.clock.clone(),
            accounts.stake_history.clone(),
            accounts.stake_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; RECLAIM_STAKE_ACCOUNT_IX_ACCOUNTS_LEN]>
    for ReclaimStakeAccountAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; RECLAIM_STAKE_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            stake_account: &arr[0],
            pool_account: &arr[1],
            pool_sol_reserves: &arr[2],
            stake_account_record_account: &arr[3],
            clock: &arr[4],
            stake_history: &arr[5],
            stake_program: &arr[6],
        }
    }
}
pub const RECLAIM_STAKE_ACCOUNT_IX_DISCM: [u8; 8] = [47, 127, 90, 221, 10, 160, 183, 117];
#[derive(Clone, Debug, PartialEq)]
pub struct ReclaimStakeAccountIxData;
impl ReclaimStakeAccountIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != RECLAIM_STAKE_ACCOUNT_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    RECLAIM_STAKE_ACCOUNT_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&RECLAIM_STAKE_ACCOUNT_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn reclaim_stake_account_ix_with_program_id(
    program_id: Pubkey,
    keys: ReclaimStakeAccountKeys,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; RECLAIM_STAKE_ACCOUNT_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: ReclaimStakeAccountIxData.try_to_vec()?,
    })
}
pub fn reclaim_stake_account_ix(keys: ReclaimStakeAccountKeys) -> std::io::Result<Instruction> {
    reclaim_stake_account_ix_with_program_id(crate::ID, keys)
}
pub fn reclaim_stake_account_invoke_with_program_id(
    program_id: Pubkey,
    accounts: ReclaimStakeAccountAccounts<'_, '_>,
) -> ProgramResult {
    let keys: ReclaimStakeAccountKeys = accounts.into();
    let ix = reclaim_stake_account_ix_with_program_id(program_id, keys)?;
    invoke_instruction(&ix, accounts)
}
pub fn reclaim_stake_account_invoke(
    accounts: ReclaimStakeAccountAccounts<'_, '_>,
) -> ProgramResult {
    reclaim_stake_account_invoke_with_program_id(crate::ID, accounts)
}
pub fn reclaim_stake_account_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: ReclaimStakeAccountAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: ReclaimStakeAccountKeys = accounts.into();
    let ix = reclaim_stake_account_ix_with_program_id(program_id, keys)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn reclaim_stake_account_invoke_signed(
    accounts: ReclaimStakeAccountAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    reclaim_stake_account_invoke_signed_with_program_id(crate::ID, accounts, seeds)
}
pub fn reclaim_stake_account_verify_account_keys(
    accounts: ReclaimStakeAccountAccounts<'_, '_>,
    keys: ReclaimStakeAccountKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.stake_account.key, keys.stake_account),
        (*accounts.pool_account.key, keys.pool_account),
        (*accounts.pool_sol_reserves.key, keys.pool_sol_reserves),
        (
            *accounts.stake_account_record_account.key,
            keys.stake_account_record_account,
        ),
        (*accounts.clock.key, keys.clock),
        (*accounts.stake_history.key, keys.stake_history),
        (*accounts.stake_program.key, keys.stake_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn reclaim_stake_account_verify_writable_privileges<'me, 'info>(
    accounts: ReclaimStakeAccountAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.stake_account,
        accounts.pool_account,
        accounts.pool_sol_reserves,
        accounts.stake_account_record_account,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn reclaim_stake_account_verify_account_privileges<'me, 'info>(
    accounts: ReclaimStakeAccountAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    reclaim_stake_account_verify_writable_privileges(accounts)?;
    Ok(())
}
pub const UNSTAKE_IX_ACCOUNTS_LEN: usize = 13;
#[derive(Copy, Clone, Debug)]
pub struct UnstakeAccounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub unstaker: &'me AccountInfo<'info>,
    pub stake_account: &'me AccountInfo<'info>,
    pub destination: &'me AccountInfo<'info>,
    pub pool_account: &'me AccountInfo<'info>,
    pub pool_sol_reserves: &'me AccountInfo<'info>,
    pub fee_account: &'me AccountInfo<'info>,
    pub stake_account_record_account: &'me AccountInfo<'info>,
    pub protocol_fee_account: &'me AccountInfo<'info>,
    pub protocol_fee_destination: &'me AccountInfo<'info>,
    pub clock: &'me AccountInfo<'info>,
    pub stake_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UnstakeKeys {
    pub payer: Pubkey,
    pub unstaker: Pubkey,
    pub stake_account: Pubkey,
    pub destination: Pubkey,
    pub pool_account: Pubkey,
    pub pool_sol_reserves: Pubkey,
    pub fee_account: Pubkey,
    pub stake_account_record_account: Pubkey,
    pub protocol_fee_account: Pubkey,
    pub protocol_fee_destination: Pubkey,
    pub clock: Pubkey,
    pub stake_program: Pubkey,
    pub system_program: Pubkey,
}
impl From<UnstakeAccounts<'_, '_>> for UnstakeKeys {
    fn from(accounts: UnstakeAccounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            unstaker: *accounts.unstaker.key,
            stake_account: *accounts.stake_account.key,
            destination: *accounts.destination.key,
            pool_account: *accounts.pool_account.key,
            pool_sol_reserves: *accounts.pool_sol_reserves.key,
            fee_account: *accounts.fee_account.key,
            stake_account_record_account: *accounts.stake_account_record_account.key,
            protocol_fee_account: *accounts.protocol_fee_account.key,
            protocol_fee_destination: *accounts.protocol_fee_destination.key,
            clock: *accounts.clock.key,
            stake_program: *accounts.stake_program.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<UnstakeKeys> for [AccountMeta; UNSTAKE_IX_ACCOUNTS_LEN] {
    fn from(keys: UnstakeKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.unstaker,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.stake_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.destination,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_sol_reserves,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.fee_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.stake_account_record_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_destination,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.clock,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.stake_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; UNSTAKE_IX_ACCOUNTS_LEN]> for UnstakeKeys {
    fn from(pubkeys: [Pubkey; UNSTAKE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            unstaker: pubkeys[1],
            stake_account: pubkeys[2],
            destination: pubkeys[3],
            pool_account: pubkeys[4],
            pool_sol_reserves: pubkeys[5],
            fee_account: pubkeys[6],
            stake_account_record_account: pubkeys[7],
            protocol_fee_account: pubkeys[8],
            protocol_fee_destination: pubkeys[9],
            clock: pubkeys[10],
            stake_program: pubkeys[11],
            system_program: pubkeys[12],
        }
    }
}
impl<'info> From<UnstakeAccounts<'_, 'info>> for [AccountInfo<'info>; UNSTAKE_IX_ACCOUNTS_LEN] {
    fn from(accounts: UnstakeAccounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.unstaker.clone(),
            accounts.stake_account.clone(),
            accounts.destination.clone(),
            accounts.pool_account.clone(),
            accounts.pool_sol_reserves.clone(),
            accounts.fee_account.clone(),
            accounts.stake_account_record_account.clone(),
            accounts.protocol_fee_account.clone(),
            accounts.protocol_fee_destination.clone(),
            accounts.clock.clone(),
            accounts.stake_program.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UNSTAKE_IX_ACCOUNTS_LEN]>
    for UnstakeAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; UNSTAKE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            unstaker: &arr[1],
            stake_account: &arr[2],
            destination: &arr[3],
            pool_account: &arr[4],
            pool_sol_reserves: &arr[5],
            fee_account: &arr[6],
            stake_account_record_account: &arr[7],
            protocol_fee_account: &arr[8],
            protocol_fee_destination: &arr[9],
            clock: &arr[10],
            stake_program: &arr[11],
            system_program: &arr[12],
        }
    }
}
pub const UNSTAKE_IX_DISCM: [u8; 8] = [90, 95, 107, 42, 205, 124, 50, 225];
#[derive(Clone, Debug, PartialEq)]
pub struct UnstakeIxData;
impl UnstakeIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != UNSTAKE_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    UNSTAKE_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&UNSTAKE_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn unstake_ix_with_program_id(
    program_id: Pubkey,
    keys: UnstakeKeys,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; UNSTAKE_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: UnstakeIxData.try_to_vec()?,
    })
}
pub fn unstake_ix(keys: UnstakeKeys) -> std::io::Result<Instruction> {
    unstake_ix_with_program_id(crate::ID, keys)
}
pub fn unstake_invoke_with_program_id(
    program_id: Pubkey,
    accounts: UnstakeAccounts<'_, '_>,
) -> ProgramResult {
    let keys: UnstakeKeys = accounts.into();
    let ix = unstake_ix_with_program_id(program_id, keys)?;
    invoke_instruction(&ix, accounts)
}
pub fn unstake_invoke(accounts: UnstakeAccounts<'_, '_>) -> ProgramResult {
    unstake_invoke_with_program_id(crate::ID, accounts)
}
pub fn unstake_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: UnstakeAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: UnstakeKeys = accounts.into();
    let ix = unstake_ix_with_program_id(program_id, keys)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn unstake_invoke_signed(
    accounts: UnstakeAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    unstake_invoke_signed_with_program_id(crate::ID, accounts, seeds)
}
pub fn unstake_verify_account_keys(
    accounts: UnstakeAccounts<'_, '_>,
    keys: UnstakeKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.payer.key, keys.payer),
        (*accounts.unstaker.key, keys.unstaker),
        (*accounts.stake_account.key, keys.stake_account),
        (*accounts.destination.key, keys.destination),
        (*accounts.pool_account.key, keys.pool_account),
        (*accounts.pool_sol_reserves.key, keys.pool_sol_reserves),
        (*accounts.fee_account.key, keys.fee_account),
        (
            *accounts.stake_account_record_account.key,
            keys.stake_account_record_account,
        ),
        (
            *accounts.protocol_fee_account.key,
            keys.protocol_fee_account,
        ),
        (
            *accounts.protocol_fee_destination.key,
            keys.protocol_fee_destination,
        ),
        (*accounts.clock.key, keys.clock),
        (*accounts.stake_program.key, keys.stake_program),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn unstake_verify_writable_privileges<'me, 'info>(
    accounts: UnstakeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.payer,
        accounts.stake_account,
        accounts.destination,
        accounts.pool_account,
        accounts.pool_sol_reserves,
        accounts.stake_account_record_account,
        accounts.protocol_fee_destination,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn unstake_verify_signer_privileges<'me, 'info>(
    accounts: UnstakeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.payer, accounts.unstaker] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn unstake_verify_account_privileges<'me, 'info>(
    accounts: UnstakeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    unstake_verify_writable_privileges(accounts)?;
    unstake_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const UNSTAKE_WSOL_IX_ACCOUNTS_LEN: usize = 14;
#[derive(Copy, Clone, Debug)]
pub struct UnstakeWsolAccounts<'me, 'info> {
    pub payer: &'me AccountInfo<'info>,
    pub unstaker: &'me AccountInfo<'info>,
    pub stake_account: &'me AccountInfo<'info>,
    pub destination: &'me AccountInfo<'info>,
    pub pool_account: &'me AccountInfo<'info>,
    pub pool_sol_reserves: &'me AccountInfo<'info>,
    pub fee_account: &'me AccountInfo<'info>,
    pub stake_account_record_account: &'me AccountInfo<'info>,
    pub protocol_fee_account: &'me AccountInfo<'info>,
    pub protocol_fee_destination: &'me AccountInfo<'info>,
    pub clock: &'me AccountInfo<'info>,
    pub stake_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UnstakeWsolKeys {
    pub payer: Pubkey,
    pub unstaker: Pubkey,
    pub stake_account: Pubkey,
    pub destination: Pubkey,
    pub pool_account: Pubkey,
    pub pool_sol_reserves: Pubkey,
    pub fee_account: Pubkey,
    pub stake_account_record_account: Pubkey,
    pub protocol_fee_account: Pubkey,
    pub protocol_fee_destination: Pubkey,
    pub clock: Pubkey,
    pub stake_program: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
}
impl From<UnstakeWsolAccounts<'_, '_>> for UnstakeWsolKeys {
    fn from(accounts: UnstakeWsolAccounts) -> Self {
        Self {
            payer: *accounts.payer.key,
            unstaker: *accounts.unstaker.key,
            stake_account: *accounts.stake_account.key,
            destination: *accounts.destination.key,
            pool_account: *accounts.pool_account.key,
            pool_sol_reserves: *accounts.pool_sol_reserves.key,
            fee_account: *accounts.fee_account.key,
            stake_account_record_account: *accounts.stake_account_record_account.key,
            protocol_fee_account: *accounts.protocol_fee_account.key,
            protocol_fee_destination: *accounts.protocol_fee_destination.key,
            clock: *accounts.clock.key,
            stake_program: *accounts.stake_program.key,
            system_program: *accounts.system_program.key,
            token_program: *accounts.token_program.key,
        }
    }
}
impl From<UnstakeWsolKeys> for [AccountMeta; UNSTAKE_WSOL_IX_ACCOUNTS_LEN] {
    fn from(keys: UnstakeWsolKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.unstaker,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.stake_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.destination,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_sol_reserves,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.fee_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.stake_account_record_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_account,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.protocol_fee_destination,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.clock,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.stake_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; UNSTAKE_WSOL_IX_ACCOUNTS_LEN]> for UnstakeWsolKeys {
    fn from(pubkeys: [Pubkey; UNSTAKE_WSOL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: pubkeys[0],
            unstaker: pubkeys[1],
            stake_account: pubkeys[2],
            destination: pubkeys[3],
            pool_account: pubkeys[4],
            pool_sol_reserves: pubkeys[5],
            fee_account: pubkeys[6],
            stake_account_record_account: pubkeys[7],
            protocol_fee_account: pubkeys[8],
            protocol_fee_destination: pubkeys[9],
            clock: pubkeys[10],
            stake_program: pubkeys[11],
            system_program: pubkeys[12],
            token_program: pubkeys[13],
        }
    }
}
impl<'info> From<UnstakeWsolAccounts<'_, 'info>>
    for [AccountInfo<'info>; UNSTAKE_WSOL_IX_ACCOUNTS_LEN]
{
    fn from(accounts: UnstakeWsolAccounts<'_, 'info>) -> Self {
        [
            accounts.payer.clone(),
            accounts.unstaker.clone(),
            accounts.stake_account.clone(),
            accounts.destination.clone(),
            accounts.pool_account.clone(),
            accounts.pool_sol_reserves.clone(),
            accounts.fee_account.clone(),
            accounts.stake_account_record_account.clone(),
            accounts.protocol_fee_account.clone(),
            accounts.protocol_fee_destination.clone(),
            accounts.clock.clone(),
            accounts.stake_program.clone(),
            accounts.system_program.clone(),
            accounts.token_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UNSTAKE_WSOL_IX_ACCOUNTS_LEN]>
    for UnstakeWsolAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; UNSTAKE_WSOL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            payer: &arr[0],
            unstaker: &arr[1],
            stake_account: &arr[2],
            destination: &arr[3],
            pool_account: &arr[4],
            pool_sol_reserves: &arr[5],
            fee_account: &arr[6],
            stake_account_record_account: &arr[7],
            protocol_fee_account: &arr[8],
            protocol_fee_destination: &arr[9],
            clock: &arr[10],
            stake_program: &arr[11],
            system_program: &arr[12],
            token_program: &arr[13],
        }
    }
}
pub const UNSTAKE_WSOL_IX_DISCM: [u8; 8] = [125, 93, 190, 135, 89, 174, 142, 149];
#[derive(Clone, Debug, PartialEq)]
pub struct UnstakeWsolIxData;
impl UnstakeWsolIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != UNSTAKE_WSOL_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    UNSTAKE_WSOL_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&UNSTAKE_WSOL_IX_DISCM)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn unstake_wsol_ix_with_program_id(
    program_id: Pubkey,
    keys: UnstakeWsolKeys,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; UNSTAKE_WSOL_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: UnstakeWsolIxData.try_to_vec()?,
    })
}
pub fn unstake_wsol_ix(keys: UnstakeWsolKeys) -> std::io::Result<Instruction> {
    unstake_wsol_ix_with_program_id(crate::ID, keys)
}
pub fn unstake_wsol_invoke_with_program_id(
    program_id: Pubkey,
    accounts: UnstakeWsolAccounts<'_, '_>,
) -> ProgramResult {
    let keys: UnstakeWsolKeys = accounts.into();
    let ix = unstake_wsol_ix_with_program_id(program_id, keys)?;
    invoke_instruction(&ix, accounts)
}
pub fn unstake_wsol_invoke(accounts: UnstakeWsolAccounts<'_, '_>) -> ProgramResult {
    unstake_wsol_invoke_with_program_id(crate::ID, accounts)
}
pub fn unstake_wsol_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: UnstakeWsolAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: UnstakeWsolKeys = accounts.into();
    let ix = unstake_wsol_ix_with_program_id(program_id, keys)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn unstake_wsol_invoke_signed(
    accounts: UnstakeWsolAccounts<'_, '_>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    unstake_wsol_invoke_signed_with_program_id(crate::ID, accounts, seeds)
}
pub fn unstake_wsol_verify_account_keys(
    accounts: UnstakeWsolAccounts<'_, '_>,
    keys: UnstakeWsolKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.payer.key, keys.payer),
        (*accounts.unstaker.key, keys.unstaker),
        (*accounts.stake_account.key, keys.stake_account),
        (*accounts.destination.key, keys.destination),
        (*accounts.pool_account.key, keys.pool_account),
        (*accounts.pool_sol_reserves.key, keys.pool_sol_reserves),
        (*accounts.fee_account.key, keys.fee_account),
        (
            *accounts.stake_account_record_account.key,
            keys.stake_account_record_account,
        ),
        (
            *accounts.protocol_fee_account.key,
            keys.protocol_fee_account,
        ),
        (
            *accounts.protocol_fee_destination.key,
            keys.protocol_fee_destination,
        ),
        (*accounts.clock.key, keys.clock),
        (*accounts.stake_program.key, keys.stake_program),
        (*accounts.system_program.key, keys.system_program),
        (*accounts.token_program.key, keys.token_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn unstake_wsol_verify_writable_privileges<'me, 'info>(
    accounts: UnstakeWsolAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.payer,
        accounts.stake_account,
        accounts.destination,
        accounts.pool_account,
        accounts.pool_sol_reserves,
        accounts.stake_account_record_account,
        accounts.protocol_fee_destination,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn unstake_wsol_verify_signer_privileges<'me, 'info>(
    accounts: UnstakeWsolAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.payer, accounts.unstaker] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn unstake_wsol_verify_account_privileges<'me, 'info>(
    accounts: UnstakeWsolAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    unstake_wsol_verify_writable_privileges(accounts)?;
    unstake_wsol_verify_signer_privileges(accounts)?;
    Ok(())
}
