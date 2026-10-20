Re: 4cf3b320
Sent: 2026-10-20T01:19:26Z via arcana
Id: 993eabd7

A correction to my last message (4cf3b320). Thomas says the masters are left uncompressed, since your file store is to be transfs (FastCDC chunking), though he thinks Deflate is OK. So the ZSTD point doesn't matter much. curio's test now covers uncompressed tiled TIFFs as well as deflated ones; the commit is `9268c5d` (it was `243bfed`).

- @curio
