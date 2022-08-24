use crate::graph_client::TradingHistoryItem;
use ethers::prelude::U256;

#[derive(Clone, Debug)]
pub struct Position {
  pub size: U256,
  pub avgEntryPrice: U256,
  pub isLong: bool,
  pub unrealizedPnl: U256,
  pub realizedPnl: U256,
}

const initial_position: Position = Position {
  size: U256::zero(),
  avgEntryPrice: U256::zero(),
  isLong: false,
  unrealizedPnl: U256::zero(),
  realizedPnl: U256::zero(),
};

pub fn aggregate(trading_history_items: &mut Vec<TradingHistoryItem>) {
  trading_history_items.sort_by(|a, b| a.id.cmp(&b.id));

  let mut currentEthPosition: Position = initial_position;
  let mut currentBtcPosition: Position = initial_position;
  let mut all_positions: Vec<Position> = vec![];

  let mut trading_history_items_iter = trading_history_items.iter();
  for item in &mut trading_history_items_iter {
    let currentPosition: &mut Position;
    if item.underlying == "lETH" {
      currentPosition = &mut currentEthPosition
    } else {
      currentPosition = &mut currentBtcPosition
    }

    if item.size == U256::zero()
      || (item.size == currentPosition.size && item.isLong != currentPosition.isLong)
    {
      currentPosition.unrealizedPnl = U256::zero();
      // TODO: fix potential overflow
      currentPosition.realizedPnl = currentPosition.realizedPnl.saturating_add(
        currentPosition
          .size
          .saturating_mul(item.price.saturating_sub(currentPosition.avgEntryPrice)),
      );

      all_positions.push((*currentPosition).clone());
      *currentPosition = initial_position;
      continue;
    }

    if currentPosition.size == U256::zero() {
      currentPosition.isLong = item.isLong;
      currentPosition.size = item.size;
      currentPosition.avgEntryPrice = item.price;
    } else {
      if currentPosition.isLong == item.isLong {
        currentPosition.size = currentPosition.size.saturating_add(item.size);
        currentPosition.avgEntryPrice = (currentPosition
          .avgEntryPrice
          .saturating_mul(currentPosition.size)
          .saturating_add(item.price.saturating_mul(item.size)))
        .div_mod(currentPosition.size.saturating_add(item.size))
        .0; // TODO: improve accuracy
            // TODO: get current price from oracle
        currentPosition.unrealizedPnl = currentPosition
          .size
          .saturating_mul(item.price.saturating_sub(currentPosition.avgEntryPrice));
      } else {
        currentPosition.size = currentPosition.size.saturating_sub(item.size);
        currentPosition.realizedPnl = item
          .size
          .saturating_mul(item.price.saturating_sub(currentPosition.avgEntryPrice));
        // TODO: get current price from oracle
        currentPosition.unrealizedPnl = currentPosition
          .size
          .saturating_sub(item.size)
          .saturating_mul(item.price.saturating_sub(currentPosition.avgEntryPrice));
      }
    }

    match (&trading_history_items).last() {
      Some(last_item) if last_item == item => all_positions.push((*currentPosition).clone()),
      Some(last_item) => (),
      None => (),
    }
  }

  println!("{:#?}", all_positions);
}
