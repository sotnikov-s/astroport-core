import {Client, executeContract, newClient, queryContract, readArtifact} from "./helpers.js";
import * as fs from "fs";
import * as util from "util";

async function pairs(cl: Client) {
    let terraSwapFactoryAddress = "terra1ulgw0td86nvs4wtpsc80thv6xelk76ut7a7apj";

    let totalPairs = [];
    let response = await queryContract(cl.terra, terraSwapFactoryAddress, {"pairs": {"limit": 30}});
    totalPairs.push(...response.pairs);

    do {
        response = await queryContract(cl.terra, terraSwapFactoryAddress, {
            "pairs": {
                "limit": 30,
                "start_after": totalPairs[totalPairs.length - 1].asset_infos
            }
        });
        totalPairs.push(...response.pairs);
    } while (response.pairs.length > 0);

    return totalPairs;
}

async function createPair(cl: Client, factoryAddr: string, pair: any) {
    return await executeContract(cl.terra, cl.wallet, factoryAddr,
        {
            'create_pair': {
                'asset_infos': pair.asset_infos,
                'pair_type': pair.pair_type,
            }
        });
}

async function main() {
    const clientLCD = newClient()
    console.log(`chainID: ${clientLCD.terra.config.chainID} wallet: ${clientLCD.wallet.key.accAddress}`)
    let network = readArtifact(clientLCD.terra.config.chainID)
    console.log('network:', network)

    let terraswap_pairs = await pairs(clientLCD);

    let pairs_list = []
    let astroportFactoryAddress = "terra1fnywlw4edny3vw44x04xd67uzkdqluymgreu7g";

    fs.writeFileSync('./terraswap_pairs.json', JSON.stringify(terraswap_pairs, null, 2) , 'utf-8');

    for(let i=0; i<terraswap_pairs.length; i++){
        try {
            await queryContract(clientLCD.terra, astroportFactoryAddress,
                {"pair": {"asset_infos": terraswap_pairs[i].asset_infos}});
        } catch (e) {
            // @ts-ignore
            console.log(e.response.data);
            pairs_list.push(terraswap_pairs[i]);
            //await createPair(clientLCD, astroportFactoryAddress, terraswap_pairs[i]);
        }
    }

    fs.writeFileSync('./non_exists_pairs.json', JSON.stringify(pairs_list, null, 2) , 'utf-8');
    console.log("size: ", pairs_list.length)
}
main().catch(console.log)
