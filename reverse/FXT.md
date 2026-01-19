# GTA 1 FXT file format

The FXT files in GTA 1 contain the text for the game.

From [gtamods wiki](https://gtamods.com/wiki/GXT#GTA_1_Format):

> GTA 1 language files have .fxt extension, and it's different from other games' format. Except the first 8 byte it uses very simple encryption. To get text, just substract 1 from every byte. First 8 bytes have different encryption, probably just to confuse modders. For every byte substract a value returned from left shifting 99.

See [`decrypt_fxt.py`](decrypt_fxt.py) for some code doing exactly that.
