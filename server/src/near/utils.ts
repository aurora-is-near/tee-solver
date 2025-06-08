import { ViewFunctionCallOptions } from 'near-api-js/lib/account';
import { getConfig } from '../config';
import { connect, keyStores, Near } from 'near-api-js';
import { parseNearAmount } from 'near-api-js/lib/utils/format';

export type NearService = {
  near: Near;
  keyStore: keyStores.InMemoryKeyStore;
};

export async function initNear(): Promise<NearService> {
  const config = await getConfig();
  const keyStore = new keyStores.InMemoryKeyStore();
  const near = await connect({
    networkId: config.near.networkId,
    nodeUrl: config.near.rpcUrl,
    keyStore,
  });
  return { near, keyStore };
}

export async function viewFunction(options: ViewFunctionCallOptions) {
  const { near } = await initNear();
  const account = await near.account("");
  return account.viewFunction(options);
}

export async function transfer(receiverId: string, amount: number) {
  const { near } = await initNear();
  const account = await near.account("");
  const amountInYocto = parseNearAmount(amount.toString());
  if (!amountInYocto) {
    throw new Error("Invalid amount");
  }
  return account.sendMoney(receiverId, BigInt(amountInYocto));
}
