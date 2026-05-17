## Experiment 2.1
![alt text](assets/experiment2.1.png)
Cara Menjalankan:

Buka satu terminal, lalu jalankan server dengan perintah: cargo run --bin server. Server akan berjalan dan mendengarkan pada port 2000.

Buka terminal baru (bisa di-split atau di tab baru), dan jalankan client pertama dengan perintah: cargo run --bin client.

Buka lagi dua terminal baru, dan jalankan client kedua dan ketiga menggunakan perintah yang sama.

Apa yang Terjadi Ketika Mengetik Pesan?
Ketika klien pertama mengetik sesuatu dan menekan Enter, input tersebut ditangkap oleh fungsi stdin.next_line() dan dikirim ke server melalui protokol WebSocket (ws_stream.send(...)).

Server menggunakan makro tokio::select! untuk menangani banyak event secara asinkron. Ketika server menerima pesan, ia mengirimkannya ke bcast_tx (Tokio Broadcast Channel). Channel ini lalu "menyiarkan" (broadcast) pesan tersebut ke semua fungsi receiver (bcast_rx) yang dimiliki klien lain, sehingga pesan tersebut muncul di layar terminal milik klien kedua dan ketiga di waktu yang hampir bersamaan.