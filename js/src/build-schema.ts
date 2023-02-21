import codegen from '@cosmwasm/ts-codegen';

for (const c of [ "price_prediction", "fast_oracle" ]) {
  console.log(`Generating types for ${c} contract`)
  codegen({
    contracts: [
      {
        name: 'price_prediction',
        dir: `../contracts/${c}/schema`,
      },
    ],
    outPath: `../contracts/${c}/ts`
  })
}
