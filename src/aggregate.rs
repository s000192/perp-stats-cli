use crate::graph_client::TradingHistoryItem;
use ethers::{prelude::I256, utils::format_units};

#[derive(Clone, Debug)]
pub struct Position {
  pub size: I256,
  pub avgEntryPrice: I256,
  pub isLong: bool,
  pub unrealizedPnl: I256,
  pub realizedPnl: I256,
  pub underlying: String,
}

#[derive(Clone, Debug)]
pub struct FormattedPosition {
  pub size: String,
  pub avgEntryPrice: String,
  pub isLong: bool,
  pub unrealizedPnl: String,
  pub realizedPnl: String,
  pub underlying: String,
}

const initial_position: Position = Position {
  size: I256::zero(),
  avgEntryPrice: I256::zero(),
  isLong: false,
  unrealizedPnl: I256::zero(),
  realizedPnl: I256::zero(),
  underlying: String::new(),
};

pub fn aggregate(
  trading_history_items: &mut Vec<TradingHistoryItem>,
  lbtc_current_price: &I256,
  leth_current_price: &I256,
) -> Option<Vec<FormattedPosition>> {
  trading_history_items.sort_by(|a, b| a.id.cmp(&b.id));

  let mut currentEthPosition: Position = initial_position;
  currentEthPosition.underlying = String::from("lETH");
  let mut currentBtcPosition: Position = initial_position;
  currentBtcPosition.underlying = String::from("lBTC");

  let mut all_positions: Vec<Position> = vec![];

  let mut trading_history_items_iter = trading_history_items.iter();
  for item in &mut trading_history_items_iter {
    // initialize currentPosition based on underlying of trading history item
    let currentPosition: &mut Position;
    if item.underlying == "lETH" {
      currentPosition = &mut currentEthPosition
    } else {
      currentPosition = &mut currentBtcPosition
    }

    // close position when
    // 1. trading history item size == 0 OR trading history item size == current position size
    // 2. trading history isLong is opposite to that of current position
    if (item.size == I256::zero() || item.size == currentPosition.size)
      && item.isLong != currentPosition.isLong
    {
      currentPosition.unrealizedPnl = I256::zero();
      // TODO: deduct platform fees
      currentPosition.realizedPnl = currentPosition.realizedPnl.checked_add(
        currentPosition
          .size
          .checked_mul(item.price.checked_sub(currentPosition.avgEntryPrice)?)?,
      )?;

      // push currentPosition values to array of all positions after calculating realized pnl
      all_positions.push((*currentPosition).clone());
      // reset currentPosition based on underlying of trading history item
      if item.underlying == "lETH" {
        *currentPosition = Position {
          size: I256::zero(),
          avgEntryPrice: I256::zero(),
          isLong: false,
          unrealizedPnl: I256::zero(),
          realizedPnl: I256::zero(),
          underlying: String::from("lETH"),
        };
      } else {
        *currentPosition = Position {
          size: I256::zero(),
          avgEntryPrice: I256::zero(),
          isLong: false,
          unrealizedPnl: I256::zero(),
          realizedPnl: I256::zero(),
          underlying: String::from("lBTC"),
        };
      }
    }

    // if position is not opened yet,
    // set values of currentPosition as that of trading history item
    if currentPosition.size == I256::zero() {
      currentPosition.isLong = item.isLong;
      currentPosition.size = item.size;
      currentPosition.avgEntryPrice = item.price;
    } else {
      // if the trading action is add position,
      // amend avg entry price and size accordingly
      if currentPosition.isLong == item.isLong {
        currentPosition.size = currentPosition.size.checked_add(item.size)?;
        currentPosition.avgEntryPrice = (currentPosition
          .avgEntryPrice
          .checked_mul(currentPosition.size)?
          .checked_add(item.price.checked_mul(item.size)?))?
        .checked_div(currentPosition.size.checked_add(item.size)?)?; // TODO: improve accuracy
      } else {
        // if the trading action is close position by amount
        // amend size and realized pnl accordingly
        currentPosition.size = currentPosition.size.checked_sub(item.size)?;
        // TDOO: deduct platform fees
        currentPosition.realizedPnl = item
          .size
          .checked_mul(item.price.checked_sub(currentPosition.avgEntryPrice)?)?;
      }
    }

    match (&trading_history_items).last() {
      // before the loop ends,
      // 1. calculate unrealized pnl
      // 2. add current eth and btc position to array of positions
      Some(last_item) if last_item == item => {
        // Check current position size > 0
        if currentEthPosition.size.gt(&I256::zero()) {
          currentEthPosition.unrealizedPnl =
            calculate_unrealized_pnl(&currentEthPosition, leth_current_price)?;
          all_positions.push((currentEthPosition).clone());
        }

        if currentBtcPosition.size.gt(&I256::zero()) {
          currentBtcPosition.unrealizedPnl =
            calculate_unrealized_pnl(&currentBtcPosition, lbtc_current_price)?;
          all_positions.push((currentBtcPosition).clone());
        }
      }
      Some(last_item) => (),
      None => (),
    }
  }

  format_positions(all_positions)
}

fn format_positions(all_positions: Vec<Position>) -> Option<Vec<FormattedPosition>> {
  let mut all_formatted_positions: Vec<FormattedPosition> = vec![];

  for position in all_positions.iter() {
    let size_sign = I256::is_positive(position.size);
    let formatted_size_value = format_units(I256::into_raw(position.size), 18).ok()?;

    let realized_pnl_sign = I256::is_positive(position.realizedPnl);
    let formatted_realized_pnl_value =
      format_units(I256::into_raw(position.realizedPnl), 36).ok()?;

    let unrealized_pnl_sign = I256::is_positive(position.unrealizedPnl);
    let formatted_unrealized_pnl_value =
      format_units(I256::into_raw(position.unrealizedPnl), 36).ok()?;

    all_formatted_positions.push(FormattedPosition {
      size: match size_sign {
        true => formatted_size_value,
        false => format!("{}{}", "-", formatted_size_value),
      },
      avgEntryPrice: format_units(I256::into_raw(position.avgEntryPrice), 18).ok()?,
      isLong: position.isLong,
      unrealizedPnl: match unrealized_pnl_sign {
        true => formatted_unrealized_pnl_value,
        false => format!("{}{}", "-", formatted_unrealized_pnl_value),
      },
      realizedPnl: match realized_pnl_sign {
        true => formatted_realized_pnl_value,
        false => format!("{}{}", "-", formatted_realized_pnl_value),
      },
      underlying: position.underlying.clone(),
    })
  }

  Some(all_formatted_positions)
}

fn calculate_unrealized_pnl(current_position: &Position, current_price: &I256) -> Option<I256> {
  // TODO: deduct platform fees
  if current_position.isLong {
    return current_position
      .size
      .checked_mul(current_price.checked_sub(current_position.avgEntryPrice)?);
  } else {
    return current_position
      .size
      .checked_mul(current_position.avgEntryPrice.checked_sub(*current_price)?);
  }
}
