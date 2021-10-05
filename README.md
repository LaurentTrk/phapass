# PhaPass, a password manager on Phala

_Work In Progress_

A project for the [Advanced Phala Challenge](https://github.com/Phala-Network/Encode-Hackathon-2021/blob/master/advanced-challenge.md) of the [Encode Polkadot Hackathon](https://www.encode.club/polkadot-club-hackathon).

It tries to demonstrate how we could rely on the Phala Confidential Contract feature to keep track of passwords.

This repository holds the PhaPass blockchain code, which implements the [PhaPass contract](./crates/phactory/src/contracts/phapass.rs).
See the [Phala instructions](./README.phala.md) on how to build and run the blockchain.

The frontend part of this project is served by a [Chrome Extension](https://developer.chrome.com/docs/extensions/) based on the [Phala JS SDK](https://github.com/Phala-Network/js-sdk). The related code can be found in [this fork](https://github.com/LaurentTrk/js-sdk).


:warning: As a Hackathon project, this is not ready for production use. Use it at your own risks :)