# Workshop Materials & Hands-On Exercises

Comprehensive workshop materials for teaching blockchain development with StrellerMinds-SmartContracts.

## Table of Contents

1. [Workshop Facilitator Guide](#workshop-facilitator-guide)
2. [Beginner Workshops](#beginner-workshops)
3. [Intermediate Workshops](#intermediate-workshops)
4. [Advanced Workshops](#advanced-workshops)
5. [Exercise Library](#exercise-library)
6. [Assessment Rubrics](#assessment-rubrics)

---

## Workshop Facilitator Guide

### Before the Workshop

**Preparation Checklist (Complete 1 Week Before):**

- [ ] Review all workshop materials thoroughly
- [ ] Test all code examples and exercises
- [ ] Prepare development environment on workshop machines
- [ ] Create participant accounts on necessary platforms
- [ ] Print workbooks and reference materials
- [ ] Prepare name tags and seating arrangement
- [ ] Test A/V equipment and screen sharing
- [ ] Setup communication channel (Discord/Slack)
- [ ] Send pre-workshop email to participants

**Technical Requirements:**

**For In-Person Workshops:**
- Projector and screen
- Whiteboard or flip charts
- Reliable WiFi (minimum 50 Mbps)
- Power strips for laptops
- Backup internet connection (mobile hotspot)

**For Virtual Workshops:**
- Zoom/Google Meet Pro account
- Screen sharing capability
- Breakout room functionality
- Recording enabled
- Co-host for technical support

**Participant Prerequisites:**

Send this email 3 days before workshop:

```
Subject: Pre-Workshop Preparation - Blockchain Development Workshop

Hi [Name],

We're excited to have you at our upcoming workshop! To make the most of your experience, please complete these steps:

REQUIRED (Complete by [DATE]):
✓ Install Rust: https://rustup.rs/
✓ Install VS Code: https://code.visualstudio.com/
✓ Create GitHub account (if you don't have one)
✓ Join our Discord: [invite link]

RECOMMENDED:
□ Watch "Blockchain Basics" video (15 min)
□ Complete Rustlings exercises 1-5
□ Read "What is Stellar?" guide

WHAT TO BRING:
- Laptop with charger
- Notebook and pen
- Questions you're excited about!

TROUBLESHOOTING:
If you encounter any issues during setup, join our tech check session:
📅 Date: [DATE]
⏰ Time: [TIME]
🔗 Link: [ZOOM LINK]

See you soon!
[Your Name]
Workshop Facilitator
```

### During the Workshop

**Time Management Tips:**

**Golden Rules:**
- Start on time, even if people are late
- Take breaks exactly as scheduled
- Have "stretch goals" for fast learners
- Prepare catch-up resources for those who fall behind

**Pacing Guidelines:**
- Lecture: 20% of time
- Demonstrations: 30% of time
- Hands-on practice: 40% of time
- Q&A: 10% of time

**Common Challenges & Solutions:**

| Challenge | Solution |
|-----------|----------|
| Participant falls behind | Assign buddy, provide recorded demo |
| Technical difficulties | Have backup laptop ready, offline materials |
| Wide skill gap | Tiered exercises (basic, intermediate, advanced) |
| Low engagement | Use polls, quizzes, pair programming |
| Too many questions | Designate "parking lot" for off-topic questions |

### After the Workshop

**Follow-Up Timeline:**

**Immediately After (Same Day):**
- Send thank you email
- Share recording links
- Post slides and materials
- Create certificate of completion

**24-48 Hours After:**
- Send feedback survey
- Answer unanswered questions
- Share additional resources
- Invite to next workshop

**1 Week After:**
- Check in on progress
- Remind about office hours
- Share community projects
- Announce next cohort

**1 Month After:**
- Request success stories
- Invite as mentor for new cohort
- Share job opportunities
- Feature standout projects

---

## Beginner Workshops

### Workshop B1: "Blockchain Fundamentals" (Half-Day, 4 hours)

**Target Audience**: Complete beginners  
**Max Participants**: 30  
**Prerequisites**: Basic computer skills  

#### Schedule

**Session 1: Welcome & Icebreaker (30 min)**
- Introductions (15 min)
- Pre-workshop poll: "What brings you here?" (10 min)
- Workshop overview and goals (5 min)

**Session 2: Blockchain Concepts (60 min)**
- Interactive lecture: What is blockchain? (30 min)
- Group activity: Centralized vs. Decentralized (20 min)
- Q&A (10 min)

**☕ BREAK (15 min)**

**Session 3: Stellar Deep Dive (60 min)**
- Presentation: Stellar ecosystem (25 min)
- Demo: Exploring Stellar blockchain (20 min)
- Hands-on: Create your first account (15 min)

**Session 4: Practical Exercise (75 min)**
- Guided: View transaction on testnet (30 min)
- Individual: Send test tokens (30 min)
- Share & Discuss (15 min)

**Closing (30 min)**
- Key takeaways (10 min)
- Next steps and resources (10 min)
- Feedback forms (10 min)

#### Materials Needed

**For Facilitator:**
- Slide deck: "Blockchain Fundamentals"
- Whiteboard markers (multiple colors)
- Printed diagrams of blockchain structure
- Sample XLM tokens for demonstration
- Certificates of completion

**For Participants:**
- Workbook with exercises
- Quick reference card
- Sticker sheet for achievements
- Pen and notebook
- Access to Stellar testnet faucet

#### Exercise 1.1: "Your First Transaction" (30 minutes)

**Learning Objective**: Successfully send a transaction on Stellar testnet

**Instructions:**

```markdown
STEP 1: Access Stellar Laboratory (10 min)
1. Open: https://laboratory.stellar.org/
2. Select "Test Network" from dropdown
3. Click "Manage Accounts"
4. Click "Generate Keypair"
5. SAVE YOUR SECRET KEY (write it down!)

STEP 2: Fund Your Account (10 min)
1. Copy your public key
2. Go to: https://friendbot.stellar.org/
3. Paste your public key
4. Click "Submit"
5. Wait for confirmation (~10 seconds)
6. Verify balance: 10,000 XLM

STEP 3: Send Payment (10 min)
1. Back to Stellar Lab
2. Go to "Build Transactions"
3. Select "Payment" operation
4. Enter recipient address
5. Enter amount: 100 XLM
6. Click "Sign in Tx Builder"
7. Paste your secret key
8. Click "Submit"
9. CELEBRATE! You just sent blockchain payment! 🎉
```

**Success Criteria:**
- ✓ Generated keypair
- ✓ Account funded with test XLM
- ✓ Successfully sent payment
- ✓ Can view transaction on Stellar Expert

**Troubleshooting:**
- If Friendbot fails: Try again, rate limited sometimes
- If transaction fails: Check sequence number
- If stuck: Raise hand, TA will help

**Extension Activity** (for fast finishers):
- Send payment to 3 classmates
- Create a multi-signature account
- Add a memo to your transaction

---

### Workshop B2: "Rust Programming Bootcamp" (Full-Day, 6 hours)

**Target Audience**: Developers with programming experience  
**Max Participants**: 20  
**Prerequisites**: Basic programming knowledge  

#### Schedule

**Morning Session (3 hours)**

**Module 1: Rust Basics (90 min)**
- Why Rust? (15 min)
- Variables and mutability (20 min)
- Data types deep dive (25 min)
- Functions and control flow (30 min)

**☕ BREAK (15 min)**

**Module 2: Ownership (75 min)**
- Stack vs. Heap (10 min)
- Ownership rules (20 min)
- References and borrowing (30 min)
- Practice exercises (15 min)

**LUNCH BREAK (30 min)**

**Afternoon Session (2.5 hours)**

**Module 3: Structs & Enums (60 min)**
- Defining structs (20 min)
- Methods (20 min)
- Enum patterns (20 min)

**Module 4: Build-a-Project (60 min)**
- Choose: Calculator OR Grade Manager
- Pair programming
- Instructor circulates for help

**Wrap-Up (30 min)**
- Project showcases
- Common challenges discussion
- Resources for continued learning

#### Hands-On Exercises

**Exercise 2.1: "Rust Syntax Sprint" (20 minutes)**

```rust
// CHALLENGE 1: Fix the errors (5 min)
fn main() {
    let x = 5;
    x = 10; // This won't compile!
    println!("x is {}", x);
}

// CHALLENGE 2: Write a function (5 min)
// Create function that takes two numbers and returns sum

// CHALLENGE 3: Pattern matching (5 min)
enum Color {
    Red,
    Green,
    Blue,
}

fn print_color(c: Color) {
    // Implement with match
}

// CHALLENGE 4: Vector manipulation (5 min)
let mut numbers = vec![1, 2, 3];
// Add 4, 5, 6
// Print all numbers
// Calculate average
```

**Exercise 2.2: "Ownership Olympics" (30 minutes)**

Team competition with 5 rounds:

**Round 1**: Identify ownership violations (5 min)
**Round 2**: Fix borrowing errors (5 min)
**Round 3**: Convert clones to references (5 min)
**Round 4**: Lifetime annotation challenge (5 min)
**Round 5**: Refactor for ownership (10 min)

Winning team gets: Rust stickers + priority registration for next workshop

---

### Workshop B3: "Smart Contract Starter" (2 Days, 12 hours total)

**Day 1**: Building Your First Contract
**Day 2**: Testing and Deployment

#### Capstone Exercise: "Hello World Contract"

**Requirements:**
```rust
// Build a contract that:
// 1. Stores a greeting message
// 2. Allows updating the greeting
// 3. Emits event when updated
// 4. Has comprehensive tests
// 5. Deploys to testnet

#[contractimpl]
impl GreetingContract {
    pub fn set_greeting(env: Env, greeting: String) {
        // Your code here
    }
    
    pub fn get_greeting(env: Env) -> String {
        // Your code here
    }
}
```

**Grading Rubric:**
- Functionality (40%): Works as specified
- Code Quality (30%): Clean, readable, follows conventions
- Tests (20%): Comprehensive test coverage
- Documentation (10%): Clear comments and README

---

## Intermediate Workshops

### Workshop I1: "DeFi Protocol Development" (2 Days)

**Prerequisites**: Completed beginner workshops or equivalent experience

#### Day 1: Architecture & Implementation

**Morning: Design Session (3 hours)**
- DeFi protocol overview (30 min)
- Tokenomics design (60 min)
- Smart contract architecture (60 min)
- Security considerations (30 min)

**Afternoon: Build Session (3 hours)**
- Token contract implementation
- Staking mechanism
- Reward distribution logic
- Integration testing

#### Day 2: Advanced Features & Launch

**Morning: Enhancement (3 hours)**
- Governance voting system
- Vesting schedules
- Emergency pause mechanism
- Upgrade patterns

**Afternoon: Production Prep (3 hours)**
- Security audit simulation
- Gas optimization
- Documentation writing
- Deployment planning

**Final Project**: Working DeFi protocol with:
- Custom token
- Staking pool
- Governance
- Full test suite

---

### Workshop I2: "Security Auditing Workshop" (Full-Day)

**Focus**: Identifying and preventing vulnerabilities

#### Morning: Attack Vectors (3 hours)

**Module 1: Common Vulnerabilities (90 min)**
- Reentrancy attacks (live demo!)
- Integer overflow/underflow
- Access control failures
- Front-running

**Module 2: Audit Tools (60 min)**
- Static analysis tools
- Fuzzing frameworks
- Symbolic execution
- Manual review checklist

#### Afternoon: Hands-On Auditing (3 hours)

**Exercise: Audit This Contract**

Provide intentionally vulnerable contract:
```rust
// THIS CONTRACT HAS MULTIPLE VULNERABILITIES
// Find as many as you can in 60 minutes!

#[contractimpl]
impl VulnerableBank {
    pub fn deposit(env: Env, amount: i128) { ... }
    pub fn withdraw(env: Env, amount: i128) { ... }
    // ... more functions
}
```

**Scoring:**
- Critical vulnerability found: 10 points
- Major vulnerability: 5 points
- Minor issue: 2 points
- Exploit demonstrated: Bonus 20 points

**Top 3 auditors win**: Security audit toolkit + certification

---

## Advanced Workshops

### Workshop A1: "Production System Architecture" (3 Days)

**Capstone Project**: Design and implement complete educational platform

**Day 1**: Requirements & Design
**Day 2**: Implementation Sprint
**Day 3**: Testing, Deployment, Presentation

**Team Roles:**
- Tech Lead
- Backend Developer
- Frontend Developer  
- DevOps Engineer
- QA Engineer

**Deliverables:**
- Architecture document
- Working prototype
- Deployment pipeline
- Monitoring dashboard
- Final presentation

---

### Workshop A2: "Hackathon: Build for Education" (Weekend, 48 hours)

**Format**: Intensive building competition

**Tracks:**
1. Learning Platform Innovation
2. Credential Verification System
3. Gamification of Education
4. Accessibility Tools
5. Analytics & Insights

**Judging Criteria:**
- Innovation (30%)
- Technical Excellence (25%)
- Educational Impact (25%)
- Presentation (20%)

**Prizes:**
- 1st Place: $2000 + incubation opportunity
- 2nd Place: $1000 + mentorship
- 3rd Place: $500 + swag pack
- People's Choice: $500

---

## Exercise Library

### Category: Beginner

**EX-B001: "Setup Verification"**
- Time: 15 minutes
- Task: Install all tools, run verification script
- Success: All checks pass

**EX-B002: "First Rust Program"**
- Time: 30 minutes
- Task: Build CLI calculator
- Success: Handles +, -, *, /

**EX-B003: "Deploy Hello World"**
- Time: 45 minutes
- Task: Deploy contract to testnet
- Success: Contract callable

### Category: Intermediate

**EX-I001: "Token Implementation"**
- Time: 2 hours
- Task: Create ERC-20 equivalent
- Success: Transfers work, events emit

**EX-I002: "Multi-Sig Wallet"**
- Time: 3 hours
- Task: 2-of-3 multisig
- Success: Requires multiple signatures

**EX-I003: "Staking Contract"**
- Time: 4 hours
- Task: Stake tokens, earn rewards
- Success: APY calculated correctly

### Category: Advanced

**EX-A001: "Cross-Chain Bridge"**
- Time: 8 hours
- Task: Lock on chain A, mint on chain B
- Success: Assets transfer securely

**EX-A002: "Flash Loan Implementation"**
- Time: 6 hours
- Task: Uncollateralized loan
- Success: Atomic execution verified

**EX-A003: "DAO Governance"**
- Time: 10 hours
- Task: Proposal and voting system
- Success: Votes execute automatically

---

## Assessment Rubrics

### Code Quality Rubric

| Criteria | Excellent (4) | Good (3) | Developing (2) | Beginning (1) |
|----------|---------------|----------|----------------|---------------|
| **Functionality** | All features work perfectly | Most features work | Some features work | Few features work |
| **Code Style** | Consistent, clean, idiomatic | Generally consistent | Inconsistent style | Poor style |
| **Testing** | >90% coverage, edge cases | >75% coverage | >50% coverage | <50% coverage |
| **Documentation** | Comprehensive, clear examples | Good documentation | Minimal docs | No documentation |
| **Security** | No vulnerabilities | Minor issues | Some concerns | Major vulnerabilities |

### Presentation Rubric

| Criteria | Excellent | Good | Developing | Needs Work |
|----------|-----------|------|------------|------------|
| **Clarity** | Easy to follow, logical | Mostly clear | Somewhat confusing | Hard to understand |
| **Depth** | Insightful analysis | Good understanding | Surface level | Misunderstandings |
| **Engagement** | Highly engaging | Moderately engaging | Low engagement | Not engaging |
| **Q&A** | Thoughtful answers | Adequate answers | Struggles to answer | Cannot answer |

### Workshop Effectiveness Survey

**Send to participants 1 week after workshop:**

```
WORKSHOP FEEDBACK SURVEY

Overall Satisfaction (1-5): ⭐⭐⭐⭐⭐

Content Quality:
- Material was relevant: Yes / Somewhat / No
- Difficulty level was: Too easy / Just right / Too hard
- Pace was: Too slow / Good / Too fast

Facilitator Effectiveness:
- Knowledgeable: 1-5
- Approachable: 1-5
- Clear explanations: 1-5

Hands-On Exercises:
- Enough practice time: Yes / No
- Exercises were helpful: 1-5
- Would like more exercises: Yes / No

Environment:
- Venue was comfortable: 1-5
- WiFi was reliable: Yes / No
- Equipment worked well: 1-5

Learning Outcomes:
I can now: (check all that apply)
□ Explain blockchain concepts
□ Set up development environment
□ Write Rust code
□ Deploy smart contracts
□ Test contracts effectively
□ Continue learning independently

Net Promoter Score:
How likely are you to recommend this workshop? (0-10)

 Biggest Takeaway: _______________

One Thing to Improve: _______________

Additional Comments: _______________
```

---

## Facilitator Resources

### Quick Reference Cards

**Card 1: Common Issues & Fixes**

```
ISSUE: Rust installation fails
FIX: Run: rustup update; rustup default stable

ISSUE: Soroban CLI not found
FIX: cargo install --locked soroban-cli

ISSUE: Transaction failed on testnet
CHECK: Sequence number, network passphrase, signatures

ISSUE: Out of disk space
FIX: cargo clean; docker system prune

ISSUE: Port already in use
FIX: lsof -ti :8000 | xargs kill -9
```

**Card 2: Timing Signals**

```
5 MIN WARNING: Hold up "5" sign
WRAP UP: Ring bell gently
BREAK TIME: Display break countdown
EMERGENCY STOP: Clap pattern (clap-clap-clapclapclap)
```

**Card 3: Engagement Boosters**

```
ENERGY LOW? → Do quick stretch break
CONFUSION SEEN? → Ask "Who needs help?"
FAST FINISHERS? → Give bonus challenge question
QUIET GROUP? → Use think-pair-share technique
DOMINATING VOICES? → "Let's hear from others"
```

---

## Budget Template

### Half-Day Workshop (30 participants)

**Venue**: $500 (if not free)
**Catering**: $600 (coffee, snacks, lunch)
**Materials**: $200 (printing, name tags, etc.)
**Swag**: $450 ($15/person for stickers, pens, notebooks)
**Facilitator Honorarium**: $1000
**TA Support**: $600 (2 TAs @ $300 each)
**Contingency**: $300

**Total**: $3,650 (~$122/person)

**Sponsorship Opportunities:**
- Title Sponsor: $2000 (logo on all materials)
- Lunch Sponsor: $800 (branded lunch)
- Swag Bag Sponsor: $500 (include their items)

---

## Scaling Strategies

### Train-the-Trainer Model

**Phase 1**: Certify 5 lead facilitators (Month 1-2)
**Phase 2**: Each certifies 5 more (Month 3-4)
**Phase 3**: Scale to 10 concurrent workshops (Month 5-6)

**Certification Requirements:**
- Complete all workshops as participant
- Co-facilitate 2 workshops
- Lead 1 workshop under observation
- Pass facilitator assessment

### Virtual Workshop Adaptation

**Best Practices:**
- Shorter sessions (90 min max, then break)
- More interactive elements (polls, chats, reactions)
- Dedicated tech support person
- Recorded sessions for review
- Virtual breakout rooms for group work

**Tools:**
- Zoom Pro with breakout rooms
- Miro board for collaboration
- Slido for Q&A
- Discord for ongoing support

---

**Version**: 1.0.0  
**Last Updated**: 2026-03-27  
**Maintained By**: Education Team  
**License**: CC BY-SA 4.0 (workshop materials)
