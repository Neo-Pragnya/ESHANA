/* ESHANA Studio: Annual and daily. Varṣaphala and pañca pakṣī. Pages register with page(); data comes from the engine (val('vedic.…')). */
page('annual', () => 'Varṣaphala', 'Time', (b) => put(b, pageHead('annual', 'Varṣaphala', 'Being built.'), note('This page is being built.')), { blurb: 'varṣaphala' }); // placeholder: replace
page('pakshi', () => 'Pañca pakṣī', 'Time', (b) => put(b, pageHead('pakshi', 'Pañca pakṣī', 'Being built.'), note('This page is being built.')), { blurb: 'pañca pakṣī' }); // placeholder: replace
