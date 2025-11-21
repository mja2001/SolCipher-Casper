import { CasperClient, Contracts, CLPublicKey, CLValueBuilder, DeployUtil, Signer } from "casper-js-sdk";

const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || "https://rpc.testnet.casper.network";
const NETWORK_NAME = "casper-test";

const client = new CasperClient(RPC_URL);

const contract = new Contracts.Contract(client);

export const contractHash = process.env.NEXT_PUBLIC_CONTRACT_HASH; // you will set this after deploy

export const connectWallet = async () => {
  await Signer.sendConnectionRequest();
};

export const isConnected = async () => Signer.isConnected();

export const getActivePublicKey = async () => Signer.getActivePublicKey();

export const signAndSendDeploy = async (deploy: any) => {
  const signed = await Signer.signMessage(deploy.toJson(), await getActivePublicKey());
  return client.putDeploy(signed);
};

export const getCidIfAllowed = async (shareId: string) => {
  if (!contractHash) throw new Error("Contract not set");
  contract.setContractHash(contractHash);
  const result = await contract.callEntrypoint(
    "get_cid_if_allowed",
    runtime_args({ share_id: CLValueBuilder.u256(shareId) }),
    CLPublicKey.fromHex(await getActivePublicKey()),
    NETWORK_NAME,
    "5000000000" // adjust gas
  );
  return result.asString();
};

export const createShare = async (cid: string, recipients: string[], expiry: number) => {
  const publicKey = await getActivePublicKey();
  const deploy = DeployUtil.makeDeploy(
    new DeployUtil.DeployParams(CLPublicKey.fromHex(publicKey), NETWORK_NAME),
    DeployUtil.ExecutableDeployItem.newStoredContractByHash(
      Buffer.from(contractHash),
      "create_share",
      runtime_args({
        cid: CLValueBuilder.string(cid),
        recipients: CLValueBuilder.list(recipients.map(r) => CLValueBuilder.key(CLPublicKey.fromHex(r))),
        expiry: CLValueBuilder.u64(expiry),
      })
    ),
    DeployUtil.standardPayment(10000000000)
  );

  const signedDeploy = await Signer.sign(deploy.toJson(), publicKey);
  const deployHash = await client.putDeploy(signedDeploy);
  return deployHash;
};

export const revokeShare = async (shareId: string) => {
  // similar to create_share but call "revoke_share"
};
