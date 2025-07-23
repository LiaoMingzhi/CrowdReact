async function placeBet() {
  try {
    // 1. 获取交易参数
    const response = await axios.get(`${API_URL}/bet/transaction-params`, {
      params: {
        from: account.value,
        amount: betAmount.value
      }
    });

    // 2. 发送交易
    const provider = new ethers.providers.Web3Provider(window.ethereum);
    const signer = provider.getSigner();
    
    // 3. 构造交易对象
    const tx = {
      to: response.data.to,
      value: response.data.value,
      data: response.data.data
    };

    // 4. 发送交易
    const txResponse = await signer.sendTransaction(tx);
    await txResponse.wait(); // 等待交易确认

    // 5. 通知后端交易已完成
    await axios.post(`${API_URL}/bet/place_bet`, {
      account_address: account.value,
      amount: betAmount.value,
      signed_transaction: txResponse.hash
    });

    message.success('下注成功！');
  } catch (error) {
    console.error('下注失败:', error);
    message.error('下注失败');
  }
} 