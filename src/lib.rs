use poker_ranking::PokerHand;

/// Given a list of poker hands, return a list of those hands which win.
///
/// Note the type signature: this function should return _the same_ reference to
/// the winning hand(s) as were passed in, not reconstructed strings which happen to be equal.
pub fn winning_hands<'a>(hands: &[&'a str]) -> Vec<&'a str> {
    let (_, mut poker_hands) = hands.iter().fold(
        (0, Vec::with_capacity(hands.len())),
        |(idx, mut phands), &handin| {
            phands.insert(idx, PokerHand::from_parse(idx, handin));
            (idx + 1, phands)
        },
    );

    poker_hands.sort_by(|a, b| b.cmp(&a));

    let mut ret = Vec::with_capacity(poker_hands.len());
    for (idx, ph) in poker_hands.iter().enumerate() {
        if idx == 0 || ph.cmp(&poker_hands[idx - 1]) == std::cmp::Ordering::Equal {
            ret.push(hands[ph.index()]);
        } else {
            break;
        }
    }
    return ret;
}

mod poker_ranking {
    use std::{collections::HashMap, ops::Deref};

    #[derive(PartialEq, Eq, Copy, Clone)]
    pub enum PokerRank {
        StraightFlush = 9,
        FourOfAKind = 8,
        FullHouse = 7,
        Flush = 6,
        Straight = 5,
        ThreeOfAKind = 4,
        TwoPair = 3,
        OnePair = 2,
        HighCard = 1,
        NotRanked = 0,
    }

    impl Ord for PokerRank {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap()
        }
    }
    impl PartialOrd for PokerRank {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some((*self as u32).cmp(&(*other as u32)))
        }
    }

    #[derive(PartialEq, Eq, Copy, Clone)]
    pub enum CardSuit {
        Spade = 4,
        Heart = 3,
        Diamond = 2,
        Club = 1,
        Joker = 0,
    }

    impl CardSuit {
        pub fn from_abbrev(ch: char) -> Self {
            match ch.to_ascii_uppercase() {
                'S' => Self::Spade,
                'C' => Self::Club,
                'D' => Self::Diamond,
                'H' => Self::Heart,
                _ => Self::Joker,
            }
        }
    }

    impl Ord for CardSuit {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    impl PartialOrd for CardSuit {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some((*other as u32).cmp(&(*self as u32)))
        }
    }

    #[derive(Clone)]
    pub struct Card {
        suit: CardSuit,
        value: u16,
    }

    pub struct PokerHand {
        id: usize,
        cards: Vec<Card>,
        rank: PokerRank,
        rank_swapped_aces: bool,
        cards_ranked: Vec<Card>,
        spares: Vec<Card>,
    }

    impl Eq for PokerHand {}

    impl PartialEq for PokerHand {
        fn eq(&self, other: &Self) -> bool {
            self.rank == other.rank
                && self
                    .cards
                    .iter()
                    .enumerate()
                    .all(|(idx, card)| card.value == other.cards.get(idx).unwrap().value)
        }
    }

    impl Ord for PokerHand {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    impl PartialOrd for PokerHand {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            if self.eq(other) {
                Some(std::cmp::Ordering::Equal)
            } else {
                if self.rank != other.rank {
                    self.rank.partial_cmp(&other.rank)
                } else {
                    match self.rank {
                        PokerRank::StraightFlush => {
                            return self.cards_ranked[0]
                                .value
                                .partial_cmp(&other.cards_ranked[0].value);
                        }
                        PokerRank::FourOfAKind => {
                            if self.cards_ranked[0].value == other.cards_ranked[0].value {
                                return self.spares[0].value.partial_cmp(&other.spares[0].value);
                            } else {
                                return self.cards_ranked[0]
                                    .value
                                    .partial_cmp(&other.cards_ranked[0].value);
                            }
                        }
                        PokerRank::FullHouse => {
                            let ord = self.cards_ranked[0]
                                .value
                                .partial_cmp(&other.cards_ranked[0].value)
                                .unwrap();

                            match ord {
                                std::cmp::Ordering::Less | std::cmp::Ordering::Greater => {
                                    return Some(ord);
                                }
                                std::cmp::Ordering::Equal => {
                                    return self.cards_ranked[3]
                                        .value
                                        .partial_cmp(&other.cards_ranked[3].value);
                                }
                            }
                        }
                        PokerRank::Flush => {
                            for (idx, card) in self.cards_ranked.iter().enumerate() {
                                let ord = card
                                    .value
                                    .partial_cmp(&other.cards_ranked[idx].value)
                                    .unwrap();
                                match ord {
                                    std::cmp::Ordering::Less | std::cmp::Ordering::Greater => {
                                        return Some(ord);
                                    }
                                    _ => {}
                                }
                            }
                            return Some(std::cmp::Ordering::Equal);
                        }

                        PokerRank::HighCard => {
                            for (idx, card) in self.spares.iter().enumerate() {
                                let ord = card.value.partial_cmp(&other.spares[idx].value).unwrap();
                                match ord {
                                    std::cmp::Ordering::Less | std::cmp::Ordering::Greater => {
                                        return Some(ord);
                                    }
                                    _ => {}
                                }
                            }
                            return Some(std::cmp::Ordering::Equal);
                        }

                        PokerRank::Straight => {
                            return self.cards_ranked[0]
                                .value
                                .partial_cmp(&other.cards_ranked[0].value);
                        }
                        PokerRank::ThreeOfAKind | PokerRank::TwoPair | PokerRank::OnePair => {
                            let mut ord = self.cards_ranked[0]
                                .value
                                .partial_cmp(&other.cards_ranked[0].value)
                                .unwrap();
                            if ord == std::cmp::Ordering::Equal && self.rank == PokerRank::TwoPair {
                                ord = self.cards_ranked[2]
                                    .value
                                    .partial_cmp(&other.cards_ranked[2].value)
                                    .unwrap();
                            }

                            match ord {
                                std::cmp::Ordering::Less | std::cmp::Ordering::Greater => {
                                    return Some(ord);
                                }
                                std::cmp::Ordering::Equal => {
                                    for (idx, card) in self.spares.iter().enumerate() {
                                        let ord = card
                                            .value
                                            .partial_cmp(&other.spares[idx].value)
                                            .unwrap();
                                        match ord {
                                            std::cmp::Ordering::Less
                                            | std::cmp::Ordering::Greater => {
                                                return Some(ord);
                                            }
                                            _ => {}
                                        }
                                    }

                                    return Some(std::cmp::Ordering::Equal);
                                }
                            }
                        }
                        _ => {
                            panic!("rank always assigned on construction")
                        }
                    }
                }
            }
        }
    }

    impl PokerHand {
        pub fn index(&self) -> usize {
            self.id
        }

        pub fn from_parse(index: usize, phand: &str) -> Self {
            let mut result = Self {
                id: index,
                rank: PokerRank::NotRanked,
                rank_swapped_aces: false,
                spares: Vec::new(),
                cards_ranked: Vec::new(),
                cards: phand
                    .split(" ")
                    .fold(Vec::with_capacity(5), |mut cards, card| {
                        cards.push(Card {
                            suit: CardSuit::from_abbrev(card.chars().last().unwrap()),
                            value: match card
                                .chars()
                                .take(card.len() - 1)
                                .collect::<String>()
                                .parse::<u16>()
                            {
                                Ok(i) => Ok(i),
                                Err(err) => {
                                    match card
                                        .chars()
                                        .take(card.len() - 1)
                                        .collect::<String>()
                                        .deref()
                                    {
                                        "A" => Ok(14_u16),
                                        "K" => Ok(13_u16),
                                        "Q" => Ok(12_u16),
                                        "J" => Ok(11_u16),
                                        _ => Err(err),
                                    }
                                }
                            }
                            .ok()
                            .unwrap(),
                        });
                        cards
                    }),
            };

            result.cards.sort_by(|a, b| a.value.cmp(&b.value));
            result.rank_hand();
            let ret = result;
            ret
        }

        fn rank_hand(&mut self) {
            let mut rank: PokerRank = PokerRank::NotRanked;
            'rankloop: for ix in (0..=9_u32).into_iter().rev() {
                match ix {
                    ix if ix == PokerRank::StraightFlush as u32 => {
                        if self.aces_swap_over(|cards| {
                            cards
                                .into_iter()
                                .all(|f| f.suit == cards.get(0).unwrap().suit)
                                && cards.into_iter().enumerate().all(|(idx, card)| {
                                    idx == 0 || card.value == cards.get(idx - 1).unwrap().value + 1
                                })
                        }) {
                            rank = PokerRank::StraightFlush;
                            break 'rankloop;
                        }
                    }
                    ix if ix == PokerRank::FourOfAKind as u32
                        || ix == PokerRank::FullHouse as u32 =>
                    {
                        if ix == PokerRank::FourOfAKind as u32 {
                            let result =
                                self.cards.iter().fold(HashMap::new(), |mut result, card| {
                                    result.entry(card.value).or_insert(Vec::new()).push(card);
                                    result
                                });

                            if result.len() == 2 {
                                let mut result = result.values().collect::<Vec<&Vec<&Card>>>();
                                result.sort_by(|&a, &b| b.len().cmp(&a.len()));

                                match result.get(0).unwrap().len() {
                                    4 => {
                                        result.get(0).unwrap().iter().for_each(|&c| {
                                            self.cards_ranked.push(c.clone());
                                        });
                                        result.get(1).unwrap().iter().for_each(|&c| {
                                            self.spares.push(c.clone());
                                        });
                                        rank = PokerRank::FourOfAKind;
                                        break 'rankloop;
                                    }
                                    3 => {
                                        result.iter().for_each(|v| {
                                            v.iter().for_each(|&c| {
                                                self.cards_ranked.push(c.clone());
                                            })
                                        });
                                        rank = PokerRank::FullHouse;
                                        break 'rankloop;
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }

                    ix if ix == PokerRank::Flush as u32 => {
                        if self
                            .cards
                            .iter()
                            .all(|c| c.suit == self.cards.get(0).unwrap().suit)
                        {
                            self.cards
                                .iter()
                                .for_each(|c| self.cards_ranked.push(c.clone()));
                            rank = PokerRank::Flush;
                            break 'rankloop;
                        }
                    }
                    ix if ix == PokerRank::Straight as u32 => {
                        if self.aces_swap_over(|cards| {
                            cards.into_iter().enumerate().all(|(idx, card)| {
                                idx == 0 || card.value == cards.get(idx - 1).unwrap().value + 1
                            })
                        }) {
                            rank = PokerRank::Straight;
                            break 'rankloop;
                        }
                    }
                    _ => {
                        //ix if ix <= PokerRank::ThreeOfAKind as u32 => todo!(), ix if ix == PokerRank::TwoPair as u32 => todo!(), // ix if ix == PokerRank::OnePair as u32 => todo!(), // ix if ix == PokerRank::HighCard as u32 => todo!(),
                        let result = self.cards.iter().fold(HashMap::new(), |mut result, card| {
                            result.entry(card.value).or_insert(Vec::new()).push(card);
                            result
                        });
                        let mut result = result.values().collect::<Vec<&Vec<&Card>>>();
                        result.sort_by(|&a, &b| b.len().cmp(&a.len()));
                        match result.len() {
                            3 => {
                                if result.get(0).unwrap().len() == 3 {
                                    // PokerRank::ThreeOfAKind
                                    result.get(0).unwrap().iter().for_each(|&c| {
                                        self.cards_ranked.push(c.clone());
                                    });
                                    result.into_iter().skip(1).for_each(|v| {
                                        v.iter().for_each(|&c| {
                                            self.spares.push(c.clone());
                                        });
                                    });
                                    rank = PokerRank::ThreeOfAKind;
                                    break 'rankloop;
                                } else {
                                    // PokerRank::TwoPair
                                    result.iter().enumerate().for_each(|(idx, &v)| match idx {
                                        0 | 1 => v.iter().for_each(|&c| {
                                            self.cards_ranked.push(c.clone());
                                        }),
                                        _ => v.iter().for_each(|&c| {
                                            self.spares.push(c.clone());
                                        }),
                                    });
                                    self.cards_ranked.sort_by(|a, b| b.value.cmp(&a.value));
                                    rank = PokerRank::TwoPair;
                                    break 'rankloop;
                                }
                            }
                            4 => {
                                result.iter().enumerate().for_each(|(idx, &v)| match idx {
                                    0 => v.iter().for_each(|&c| {
                                        self.cards_ranked.push(c.clone());
                                    }),
                                    _ => v.iter().for_each(|&c| {
                                        self.spares.push(c.clone());
                                    }),
                                });
                                rank = PokerRank::OnePair;
                                break 'rankloop;
                            }
                            _ => {
                                result.iter().for_each(|&v| {
                                    v.iter().for_each(|&c| self.spares.push(c.clone()))
                                });
                                rank = PokerRank::HighCard;
                                break 'rankloop;
                            }
                        }
                    }
                }
            }

            if self.spares.len() > 0 {
                if self.spares.iter().any(|c| c.value == 14) {
                    self.spares.iter_mut().for_each(|c| {
                        if c.value == 14 {
                            c.value = 1
                        }
                    });
                }
                self.spares.sort_by(|a, b| b.value.cmp(&a.value));
            }

            self.rank = rank;
        }

        fn aces_swap_over(&mut self, cardcheck: impl Fn(&[Card]) -> bool) -> bool {
            let mut aswap = false;
            let mut ret: bool;
            let mut cards = self.cards.clone();

            loop {
                ret = cardcheck(cards.as_slice());
                if ret || aswap {
                    break;
                } else {
                    aswap = true;
                    cards.iter_mut().for_each(|card| {
                        if card.value == 14 {
                            card.value = 1;
                        }
                    });
                    cards.sort_by(|a, b| a.value.cmp(&b.value));
                }
            }

            if ret {
                self.cards_ranked = cards;
                self.rank_swapped_aces = aswap;
            }

            return ret;
        }
    }
}
