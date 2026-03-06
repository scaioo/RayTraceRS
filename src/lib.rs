/*
Dalla teoria per l'uso di lib.rs: 
Tipicamente, i pacchetti che contengono sia una libreria (lib.rs) che un crate
binario (main.rs) avranno nel crate binario il codice strettamente necessario 
ad avviare un eseguibile che chiama il codice definito nel crate libreria. 
Questo consente ad altri progetti di beneficiare della maggior parte 
delle funzionalità che il pacchetto fornisce, poiché il codice 
del crate libreria può essere condiviso.

L’albero dei moduli dovrebbe essere definito in src/lib.rs. 
Quindi, qualsiasi elemento pubblico può essere utilizzato 
nel crate binario facendo iniziare i path con il nome del pacchetto.
Il crate binario diventa un utilizzatore del crate libreria 
proprio come un crate completamente esterno utilizzerebbe il crate libreria: 
può utilizzare solo l’API pubblica. Questo ti aiuta a progettare una buona API; 
non solo sei l’autore, ma sei anche un cliente!

vd https://nixxo.github.io/rust-lang-book-it/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html
 */
// Mi raccomando: note per l'uso da eliminare successivamente!

