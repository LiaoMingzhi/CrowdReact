const LuckGame = artifacts.require("LuckGame");

contract("LuckGame", accounts => {
  let luckGame;
  const owner = accounts[0];
  const player = accounts[1];
  const agent = accounts[2];

  beforeEach(async () => {
    luckGame = await LuckGame.new({ from: owner });
  });

  it("should allow placing bets", async () => {
    const betAmount = web3.utils.toWei("0.1", "ether");
    await luckGame.placeBet(50, { 
      from: player, 
      value: betAmount 
    });
    
    const balance = await luckGame.getBalance(player);
    assert.equal(balance.toString(), betAmount);
  });
});