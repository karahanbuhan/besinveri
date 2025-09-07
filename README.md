# BesinVeri
Türkiye'deki gıdaların kalori, makro, mineral ve vitamin değerlerini sunan açık ve ücretsiz bir API. 

#### Şu anda geliştirme aşamasındadır, API aktif olduğunda burası güncellenecektir.

## Kurulum ve Çalıştırma
1. Öncelikle kaynak kodunu bilgisayarınıza indirin: `git clone https://github.com/karahanbuhan/besinveri.git`
2. Daha sonrasında kaynak kodunun olduğu ana klasöre girin: `cd besinveri`
3. En hızlı şekilde çalıştırmak için `cargo run` komutunu kullanabilirsiniz.

### Docker Konteyner Olarak Çalıştırma
1. Öncelikle kaynak kodunu bilgisayarınıza indirin: `git clone https://github.com/karahanbuhan/besinveri.git`
2. Daha sonrasında kaynak kodunun olduğu ana klasöre girin: `cd besinveri`
3. Docker Image oluşturmak ve bunu Konteyner olarak çalıştırmak için `docker build -t besinveri . && docker run -it besinveri`

***Dikkat:** Docker ile çalıştırıldığında `cargo run --release` kullanıldığı için inşaa süresi ekstra uzun olabilir, bundan dolayı sadece arayüzün son kullanıcıya sunulduğu ortamda bu şekilde çalıştırılması önerilir.*

