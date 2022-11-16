use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::state::{
    GameInfo, COMPLETED_GAME_LIST, GAME_LIST, LAST_GAME_ID, NEW_GAME_LIST, PLAYING_GAME_LIST,
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult,
    Uint128,
};
use cw_storage_plus::Bound;
use tic_tac_toe::tic_tac_toe::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

//Initialize the contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    LAST_GAME_ID.save(deps.storage, &0)?;
    Ok(Response::new())
}

//Execute the handle messages.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::NewGame {} => execute_new_game(deps, env, info),
        ExecuteMsg::JoinGame { game_id } => execute_join_game(deps, env, info, game_id),
        ExecuteMsg::Play {
            game_id,
            x_pos,
            y_pos,
        } => execute_play_game(deps, env, info, game_id, x_pos, y_pos),
    }
}

pub fn execute_new_game(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
    let mut last_game_id = LAST_GAME_ID.load(deps.storage)?;
    let new_game = GameInfo {
        hoster: info.sender,
        joiner: Addr::unchecked(""),
        turn: 0,
        board: [[0; 3]; 3],
        winner: 0,
    };

    GAME_LIST.save(deps.storage, last_game_id, &new_game)?;
    NEW_GAME_LIST.save(deps.storage, last_game_id, &true)?;

    last_game_id += 1;
    LAST_GAME_ID.save(deps.storage, &last_game_id)?;

    Ok(Response::new())
}

pub fn execute_join_game(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    game_id: Uint128,
) -> StdResult<Response> {
    let last_game_id = LAST_GAME_ID.load(deps.storage)?;
    if game_id.u128() >= last_game_id {
        return Err(StdError::generic_err("Invalid Game ID"));
    }

    if let Some(is_available) = NEW_GAME_LIST.may_load(deps.storage, game_id.u128())? {
        if is_available {
            let mut game_info = GAME_LIST.may_load(deps.storage, game_id.u128())?.unwrap();
            if game_info.hoster == info.sender {
                return Err(StdError::generic_err("You can't join the game you hosted."));
            }
            game_info.joiner = info.sender;

            let hash_string = game_info.hoster.to_string() + &game_info.joiner.to_string();
            let mut hasher = DefaultHasher::new();
            hash_string.hash(&mut hasher);
            let hash = hasher.finish().reverse_bits();
            if hash % 2 == 0 {
                game_info.turn = 1;
            } else {
                game_info.turn = 2;
            }

            GAME_LIST.save(deps.storage, game_id.u128(), &game_info)?;
            NEW_GAME_LIST.remove(deps.storage, game_id.u128());
            PLAYING_GAME_LIST.save(deps.storage, game_id.u128(), &true)?;
        } else {
            return Err(StdError::generic_err(
                "The game is already in progress or completed.",
            ));
        }
    } else {
        return Err(StdError::generic_err(
            "The game is already in progress or completed.",
        ));
    }
    Ok(Response::new())
}

pub fn execute_play_game(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    game_id: Uint128,
    x_pos: u16,
    y_pos: u16,
) -> StdResult<Response> {
    let last_game_id = LAST_GAME_ID.load(deps.storage)?;

    if game_id.u128() >= last_game_id {
        return Err(StdError::generic_err("Invalid Game ID"));
    }

    if x_pos >= 3 || y_pos >= 3 {
        return Err(StdError::generic_err("Invalid Position"));
    }

    if let Some(is_available) = PLAYING_GAME_LIST.may_load(deps.storage, game_id.u128())? {
        if is_available {
            let mut game_info = GAME_LIST.may_load(deps.storage, game_id.u128())?.unwrap();
            if (game_info.hoster == info.sender && game_info.turn == 2)
                || (game_info.joiner == info.sender && game_info.turn == 1)
            {
                return Err(StdError::generic_err("It's not your turn."));
            }
            if game_info.board[x_pos as usize][y_pos as usize] != 0 {
                return Err(StdError::generic_err("This position was already filled."));
            }
            game_info.board[x_pos as usize][y_pos as usize] = game_info.turn;

            let winner = calculate_winner(game_info.board, x_pos as usize, y_pos as usize);
            if winner == 0 {
                if game_info.turn == 1 {
                    game_info.turn = 2;
                } else {
                    game_info.turn = 1;
                }
            } else {
                game_info.turn = 0;
                game_info.winner = winner;
                PLAYING_GAME_LIST.remove(deps.storage, game_id.u128());
                COMPLETED_GAME_LIST.save(deps.storage, game_id.u128(), &true)?;
            }

            GAME_LIST.save(deps.storage, game_id.u128(), &game_info)?;
        } else {
            return Err(StdError::generic_err("The game is not in progress now."));
        }
    } else {
        return Err(StdError::generic_err("The game is not in progress now."));
    }
    Ok(Response::new())
}

pub fn calculate_winner(board: [[u16; 3]; 3], x_pos: usize, y_pos: usize) -> u16 {
    // check row
    if board[x_pos][0] != 0
        && board[x_pos][0] == board[x_pos][1]
        && board[x_pos][1] == board[x_pos][2]
    {
        return board[x_pos][0];
    }

    // check column
    if board[0][y_pos] != 0
        && board[0][y_pos] == board[1][y_pos]
        && board[1][y_pos] == board[2][y_pos]
    {
        return board[0][y_pos];
    }

    // check diagonal
    if x_pos == y_pos
        && board[0][0] != 0
        && board[0][0] == board[1][1]
        && board[1][1] == board[2][2]
    {
        return board[0][0];
    }
    if x_pos + y_pos == 2
        && board[0][2] != 0
        && board[0][2] == board[1][1]
        && board[1][1] == board[2][0]
    {
        return board[0][2];
    }

    // check is full
    for i in 0..2 {
        for j in 0..2 {
            if board[i][j] == 0 {
                return 0;
            }
        }
    }

    3
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GameInfoById { game_id } => to_binary(&query_game_info_by_id(deps, game_id)?),
        QueryMsg::NewGames { start_after, limit } => {
            to_binary(&query_new_games(deps, start_after, limit)?)
        }
        QueryMsg::PlayingGames { start_after, limit } => {
            to_binary(&query_playing_games(deps, start_after, limit)?)
        }
        QueryMsg::CompletedGames { start_after, limit } => {
            to_binary(&query_completed_games(deps, start_after, limit)?)
        }
    }
}

pub fn query_game_info_by_id(deps: Deps, game_id: Uint128) -> StdResult<GameInfo> {
    if let Some(game_info) = GAME_LIST.may_load(deps.storage, game_id.u128())? {
        Ok(game_info)
    } else {
        Err(StdError::generic_err("Invalid Game ID"))
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_new_games(
    deps: Deps,
    start_after: Option<Uint128>,
    limit: Option<u32>,
) -> StdResult<Vec<Uint128>> {
    let mut new_game_id_list: Vec<Uint128> = vec![];

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    NEW_GAME_LIST
        .range(
            deps.storage,
            start_after.map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|item| {
            let (k, _) = item.unwrap();
            k
        })
        .for_each(|item| new_game_id_list.append(&mut vec![Uint128::from(item)]));

    Ok(new_game_id_list)
}

pub fn query_playing_games(
    deps: Deps,
    start_after: Option<Uint128>,
    limit: Option<u32>,
) -> StdResult<Vec<Uint128>> {
    let mut playing_game_id_list: Vec<Uint128> = vec![];

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    PLAYING_GAME_LIST
        .range(
            deps.storage,
            start_after.map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|item| {
            let (k, _) = item.unwrap();
            k
        })
        .for_each(|item| playing_game_id_list.append(&mut vec![Uint128::from(item)]));

    Ok(playing_game_id_list)
}

pub fn query_completed_games(
    deps: Deps,
    start_after: Option<Uint128>,
    limit: Option<u32>,
) -> StdResult<Vec<Uint128>> {
    let mut completed_game_id_list: Vec<Uint128> = vec![];

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    COMPLETED_GAME_LIST
        .range(
            deps.storage,
            start_after.map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|item| {
            let (k, _) = item.unwrap();
            k
        })
        .for_each(|item| completed_game_id_list.append(&mut vec![Uint128::from(item)]));

    Ok(completed_game_id_list)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
