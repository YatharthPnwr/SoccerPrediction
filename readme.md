# ⚽ Kicker - Decentralized Soccer Score Prediction Platform

A Solana blockchain-based betting platform that allows users to predict and bet on sports match outcomes using an automated market maker (AMM) with dynamic odds that adjust in real-time based on live scores.

## 🎯 What It Does

Kicker is a decentralized prediction market where:

- Users bet on match outcomes using SOL
- Odds dynamically change based on live game scores via an oracle
- Winners receive proportional rewards from a prize pool
- Platform uses a constant product AMM (x\*y=k) for fair pricing
- 5% platform fee on total deposits

## 🏗️ Architecture

### Smart Contract Features

- **Dynamic Odds**: Multipliers (60%-140%) adjust virtual liquidity pools based on goal difference
- **AMM Model**: Constant product curve determines share prices
- **Security**: PDA-based vault, admin/oracle role separation, anti-cheat mechanisms

### Tech Stack

- **Framework**: Anchor 0.32.1
- **Blockchain**: Solana
- **Language**: Rust
- **Testing**: TypeScript, Mocha, Chai
- **AMM**: constant-product-curve library

## 🚀 Getting Started

### Installation

1. **Clone the repository**

```bash
git clone https://github.com/YatharthPnwr/SoccerPrediction
cd SoccerPrediction
```

2. **Build the program**

```bash
anchor build
```

3. **Run tests**

```bash
anchor test
```

## 🧪 Testing

The test suite covers:

- Match initialization and lifecycle
- Multiple user deposits and share calculations
- Oracle score updates with multiplier effects
- Reward distribution for winners/losers
- Edge cases and unauthorized access attempts

## 📁 Project Structure

```
Kicker/
├── programs/score-prediction/
│   └── src/
│       ├── lib.rs              # Program entry point
│       ├── instructions/       # All program instructions
│       └── state/              # State management
├── tests/
│   └── score-prediction.ts     # Comprehensive test suite
├── Anchor.toml                 # Anchor configuration
└── package.json                # Dependencies
```

## 📝 Devnet Program ID

```
5vNvacWDvy6asU5gM8Av1VZHT8NGM2gJE6QAqQ9DzFKC
```

## 🤝 Contributing

This is a capstone project for Turbin3. For issues or suggestions, please open an issue in the repository.
