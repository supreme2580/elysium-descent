# Gameplay

This document details the core gameplay loop and mechanics of *Elysium Descent*. It covers the two distinct phases of gameplay: the real-time Monster Survival and the turn-based On-Chain Combat in the Loot Survivor dungeons.

## Core Gameplay Loop

The core gameplay loop of *Elysium Descent* revolves around a cycle of survival and progression:

1.  **Real-time Monster Survival:** Players engage in dynamic combat against waves of increasingly challenging monsters in a procedurally generated environment. The goal is to survive for a specific duration, reach an experience threshold, or complete other defined objectives.
2.  **Objective Completion & Transition:** Upon successfully completing the survival objective, the player earns the opportunity to access a "Loot Survivor" dungeon. This transition signifies a shift in gameplay style and stakes.
3.  **Turn-Based On-Chain Combat:** Inside the Loot Survivor dungeon, players encounter a unique, on-chain generated boss. Combat is turn-based, requiring strategic decision-making regarding attacks, defense, and skill usage.
4.  **Loot and Progression:** Defeating the on-chain boss rewards the player with loot and potentially collectible beasts, and meta-progression elements that can influence future survival runs or character capabilities.
5.  **Repeat:** The player then returns to the real-time survival phase, now potentially stronger or with new tools, to face new challenges and earn access to further Loot Survivor dungeons.

## Real-time Monster Survival

This phase focuses on dynamic movement, strategic positioning, and real-time combat against hordes of enemies.

### Player Controls

* **Movement:** [WASD, joystick]. Describe any nuances like dashing or special movement abilities.
* **Attack:** [Mouse click, KeyCode::F]. Detail the basic attack mechanics, range, and any cooldowns.
* **Skills/Abilities:** [KeyCode::Q]. Describe the initial set of skills and how they function (e.g., area-of-effect damage, crowd control, defensive buffs).
* **Interaction:** [Keycode::E]. Describe how players interact with the environment (e.g., picking up items).

### Enemies

* **Enemy Variety:** (e.g., melee attackers, ranged attackers, fast movers, heavy units).
* **AI Behavior:** pathfinding.
* **Scaling Difficulty:** (e.g., more numerous enemies, tougher enemy types, special enemy waves).

### Objectives

* **Survival Time:** Players must survive for a specific duration (e.g., 5 minutes). The timer will be clearly displayed.
* **Experience Threshold:** Players need to accumulate a certain amount of experience points by defeating enemies.
* **Special Conditions:** Introduce potential variations in objectives (e.g., defeat a mini-boss, protect a specific point).

### Items and Power-ups (During Survival)

* **Pickups:** (e.g., temporary damage boosts, speed increases, health regeneration).
* **Acquisition:** (e.g., dropped by enemies, found in the environment).
* **Duration/Effect:** *Detail the duration and effects of these temporary benefits.

## Turn-Based On-Chain Combat (Loot Survivor Dungeons)

This phase shifts to a strategic, turn-based encounter against a unique boss generated and managed on the blockchain.

### Transition to Dungeons

* **Activation:** This might involve interacting with a specific point or a UI prompt.
* **On-Chain Interaction:** Uses `bevy_dojo` i.e. torii, controller & `starknet-rs`.

### Combat Mechanics

* **Turn Order:** (e.g., player always goes first).
* **Player Actions:**
* **Boss Actions:** (e.g., pre-set patterns at spawn).
* **Skills and Abilities (Player & Boss):** Skills gained during meta-progression influence and environment influence abilities and skills. There is a variety and unique nature of boss abilities.
* **Defense/Mitigation:** Based on player choices at spawn and inventory items.

### On-Chain Bosses

* **Uniqueness:**
* **Difficulty:** boss difficulty will vary, potentially influenced by factors like the player's survival performance or the dungeon's properties.

### Rewards and Loot

* **On-Chain Loot:**
* **Meta-Progression:** Explain how defeating bosses can contribute to permanent character growth or unlock new possibilities for future runs (e.g., new skills, starting items, stat increases).
* **Loot Acquisition:** Detail how the loot is awarded to the player upon defeating the boss and how it is managed on-chain.

## Meta-Progression

* **Permanent Upgrades:** Describe any permanent upgrades or unlocks players can earn that persist between runs (e.g., new skills, stat bonuses, starting equipment).
* **Unlocks:** Explain if there are any other persistent unlocks, such as new areas, enemy types, or gameplay modifiers.
* **Connection to On-Chain Loot:** Clarify how on-chain loot might contribute to or interact with the meta-progression system.

## Difficulty Scaling

* **Run-Based Difficulty:** Explain how the difficulty of the survival phases might increase with each subsequent run (e.g., tougher starting enemies, faster scaling).
* **Dungeon Difficulty:** Hint at potential variations in the difficulty of Loot Survivor dungeons.

## Death and Game Over

* **Consequences of Death (Survival):** .
* **Consequences of Failure (Dungeon):** .

This `gameplay.md` document provides a detailed overview of how *Elysium Descent* is played.
