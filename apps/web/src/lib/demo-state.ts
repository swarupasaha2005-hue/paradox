// Illustrative interface data only. No value below comes from the chain.
// Display amounts are whole example units; on-chain amounts remain integer token units.
export const demo = {
  memberName: "Rahul",
  cycle: 1,
  currentRound: 1,
  totalRounds: 10,
  members: 10,
  contribution: 5_000,
  pot: 50_000,
  contributionPreview: {
    pending: { membersPaid: 9, collected: 45_000, myStatus: "NOT PAID" },
    ready: { membersPaid: 10, collected: 50_000, myStatus: "PAID" },
    members: ["Rahul", "Riya", "Aman", "Meera", "Dev", "Leela", "Kabir", "Sana", "Arjun", "You"],
  },
  winningPayout: 43_000,
  discount: 7_000,
  discountPerMember: 700,
  commitmentPreview: "0x8f3a…91cd",
  bids: [
    { name: "Rahul", amount: 43_000 },
    { name: "Riya", amount: 46_000 },
    { name: "Aman", amount: 45_000 },
  ],
  history: {
    contributionsCompleted: 10,
    contributionsExpected: 10,
    totalContributed: 50_000,
    payoutsReceived: 1,
    totalPayoutReceived: 43_000,
    postPayoutContributionsCompleted: 8,
    postPayoutContributionsExpected: 8,
    discountsEarned: 6_700,
    discountsClaimed: 0,
    cyclesJoined: 1,
    cyclesCompleted: 1,
  },
} as const;

export const formatAmount = (value: number) => `₹${value.toLocaleString("en-IN")}`;
