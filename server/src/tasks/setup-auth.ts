import { PhalaCloudService } from "../phala/cvms";

export async function setupAuth() {
  const phala = new PhalaCloudService();
  await phala.setupPhalaAuth();
}
