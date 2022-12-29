
console.log('importing pkg...');

import('./pkg')
    .then(wasm => {
        console.log('done pkg');
    })
    .catch(console.error);
