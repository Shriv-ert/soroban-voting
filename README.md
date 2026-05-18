# Stellar Secure Voting DApp

**Stellar Secure Voting DApp** — Blockchain-Based Tamper-Proof Decentralized Voting System

## Project Description

Stellar Secure Voting DApp is a decentralized smart contract solution built on the Stellar blockchain using the Soroban SDK. It provides a secure, transparent, and tamper-proof platform for conducting elections and polls directly on the blockchain. The contract ensures that every vote is authentic, immutable, and verifiable — eliminating the risks of fraud, double-voting, and unauthorized data manipulation.

The system implements a role-based architecture with an admin who manages the election lifecycle and voters who cast their ballots. Each vote is permanently recorded on the blockchain with cryptographic authentication, ensuring complete integrity of the electoral process.

## Project Vision

Our vision is to revolutionize democratic processes in the digital age by:

- **Eliminating Fraud**: Making vote tampering mathematically impossible through blockchain immutability
- **Ensuring Transparency**: Every vote and action is publicly verifiable on the Stellar blockchain
- **Preventing Double-Voting**: Each address can only cast one vote, enforced at the smart contract level
- **Guaranteeing Authenticity**: All operations require cryptographic authentication via `require_auth()`
- **Building Trustless Elections**: Creating a system where election integrity is guaranteed by code, not by institutions

We envision a future where every election — from student council votes to national referendums — can be conducted with complete transparency and zero trust assumptions.

## Key Features

### 1. **Admin-Controlled Election Management**

- One-time initialization — admin cannot be changed after setup
- Only admin can add or remove candidates
- Admin controls the voting lifecycle (open/close)
- Admin identity is cryptographically verified on every action

### 2. **Phased Voting Lifecycle**

- **NotStarted** — Admin prepares candidates. No voting allowed
- **Open** — Voters can cast their ballots. Candidate list is locked
- **Closed** — Voting ends. Results are final and immutable
- Strict phase enforcement prevents out-of-order operations

### 3. **Anti-Fraud Protection**

- One vote per address — double-voting is impossible
- Voter status is stored in persistent storage and cannot be reset
- Candidates cannot be added or removed after voting begins
- No function exists to modify or delete cast votes

### 4. **Cryptographic Authentication**

- Every sensitive operation requires `require_auth()` signature verification
- Admin functions verify both authentication and authorization
- Voter identity is cryptographically bound to their vote record
- Impersonation attacks are prevented at the protocol level

### 5. **Stellar Network Integration**

- Leverages the high speed and low cost of the Stellar network
- Built using the modern Soroban Smart Contract SDK v25
- All data is publicly auditable on the Stellar blockchain explorer
- Interoperable with other Stellar-based governance systems

## Contract Functions

### Admin Functions

| Function | Description | Phase Required |
|----------|-------------|----------------|
| `initialize(admin)` | Set up the contract with an admin address. Can only be called once. | — |
| `add_candidate(admin, name)` | Add a new candidate to the election. | NotStarted |
| `remove_candidate(admin, candidate_id)` | Remove a candidate from the election. | NotStarted |
| `open_voting(admin)` | Open the voting period. Requires at least 2 candidates. | NotStarted |
| `close_voting(admin)` | Close the voting period. Results become final. | Open |

### Voter Functions

| Function | Description | Phase Required |
|----------|-------------|----------------|
| `vote(voter, candidate_id)` | Cast a vote for a candidate. Each address can only vote once. | Open |

### Public Functions (Read-Only)

| Function | Description |
|----------|-------------|
| `get_candidates()` | Retrieve all candidates with their vote counts. |
| `get_status()` | Get the current voting phase (NotStarted/Open/Closed). |
| `has_voted(voter)` | Check if a specific address has already voted. |
| `get_total_voters()` | Get the total number of votes cast. |
| `get_admin()` | Get the admin address of the contract. |

## Security Architecture

| Threat | Protection | Implementation |
|--------|------------|----------------|
| Unauthorized admin | One-time initialization | `initialize()` checks `has(&DataKey::Admin)` |
| Non-admin modification | Role-based access control | `require_admin()` verifies caller identity |
| Double voting | Permanent voter tracking | `DataKey::HasVoted(address)` in persistent storage |
| Premature voting | Phase enforcement | `VotingStatus` check on every operation |
| Candidate tampering | Phase locking | Candidates locked after `open_voting()` |
| Result manipulation | No edit functions | No function exists to modify cast votes |
| Identity spoofing | Cryptographic auth | `require_auth()` on all sensitive functions |

## Technical Requirements

- Soroban SDK v25
- Rust programming language
- Stellar blockchain network (Testnet or Mainnet)
- Stellar CLI (`stellar`)

## Getting Started

### Using Soroban Studio (Recommended — No Installation Required)

1. Open [soroban.studio](https://soroban.studio)
2. Replace the contents of `lib.rs` and `test.rs` with the files from this project
3. Follow the deployment steps below

### Deploy to Testnet

```bash
# 1. Create admin wallet
stellar keys generate admin --network testnet --fund

# 2. Build the smart contract
stellar contract build

# 3. Deploy to testnet
stellar contract deploy --source-account admin
```

### Interact with the Contract

```bash
# Initialize the contract
stellar contract invoke --id <CONTRACT_ID> --source-account admin -- initialize --admin admin

# Add candidates
stellar contract invoke --id <CONTRACT_ID> --source-account admin -- add_candidate --admin admin --name "Alice"
stellar contract invoke --id <CONTRACT_ID> --source-account admin -- add_candidate --admin admin --name "Bob"

# Open voting
stellar contract invoke --id <CONTRACT_ID> --source-account admin -- open_voting --admin admin

# Create voter wallets
stellar keys generate voter1 --network testnet --fund

# Cast votes (use candidate ID from get_candidates output)
stellar contract invoke --id <CONTRACT_ID> --source-account voter1 -- vote --voter voter1 --candidate_id <CANDIDATE_ID>

# View results
stellar contract invoke --id <CONTRACT_ID> --source-account admin -- get_candidates
stellar contract invoke --id <CONTRACT_ID> --source-account admin -- get_total_voters

# Close voting
stellar contract invoke --id <CONTRACT_ID> --source-account admin -- close_voting --admin admin
```

## Project Structure

```
soroban-voting/
├── Cargo.toml                          # Workspace configuration
├── README.md                           # Project documentation
└── contracts/
    └── voting/
        ├── Cargo.toml                  # Contract dependencies
        ├── Makefile                    # Build commands
        └── src/
            ├── lib.rs                  # Smart contract implementation
            └── test.rs                 # 12 comprehensive test cases
```

## Test Coverage

The project includes 12 test cases covering all contract functions and security scenarios:

| # | Test | Validates |
|---|------|-----------|
| 1 | `test_initialize` | Admin, status, and total voters are stored correctly |
| 2 | `test_cannot_initialize_twice` | Contract cannot be re-initialized |
| 3 | `test_add_candidate` | Admin can add candidates |
| 4 | `test_non_admin_cannot_add_candidate` | Non-admin users are rejected |
| 5 | `test_cannot_add_candidate_after_open` | Candidates are locked after voting opens |
| 6 | `test_open_voting` | Voting can be opened with 2+ candidates |
| 7 | `test_cannot_open_with_less_than_2` | Minimum 2 candidates required |
| 8 | `test_vote_success` | Votes are recorded correctly |
| 9 | `test_cannot_vote_twice` | Double-voting is prevented |
| 10 | `test_cannot_vote_before_open` | Voting before open phase is rejected |
| 11 | `test_cannot_vote_after_close` | Voting after close phase is rejected |
| 12 | `test_full_voting_flow` | End-to-end simulation with 5 voters and 3 candidates |

## Future Scope

### Short-Term Enhancements

1. **Voter Registration**: Implement a whitelist system where only pre-approved addresses can vote
2. **Voting Deadline**: Add time-based automatic closing using blockchain timestamps
3. **Vote Weight**: Support weighted voting for governance token holders
4. **Election Title**: Add metadata (title, description) to each election

### Medium-Term Development

5. **Multi-Election Support**: Allow multiple concurrent elections within one contract
   - Each election has its own candidates and voters
   - Independent lifecycle management
   - Cross-election analytics
6. **Delegate Voting**: Allow voters to delegate their vote to another address
7. **Result Certification**: Automatic winner declaration when voting closes
8. **Event Logging**: Emit blockchain events for real-time vote tracking

### Long-Term Vision

9. **Cross-Chain Voting**: Extend voting to multiple blockchain networks
10. **Zero-Knowledge Proofs**: Implement private voting where votes are verified but not revealed
11. **DAO Integration**: Use voting results to automatically execute governance proposals
12. **Quadratic Voting**: Implement quadratic voting mechanisms for fairer representation
13. **Frontend DApp**: Build a web interface for non-technical users
14. **Mobile Integration**: Develop mobile SDKs for voting on-the-go

### Enterprise Features

15. **Corporate Governance**: Adapt for shareholder voting and board elections
16. **Audit Trail**: Comprehensive logging for regulatory compliance
17. **Multi-Signature Admin**: Require multiple admins to approve election changes
18. **API Gateway**: REST API bridge for integration with existing systems

---

**Stellar Secure Voting DApp** — Transparent Elections, Powered by Blockchain 🗳️
