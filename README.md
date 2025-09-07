# BesinVeri
Türkiye'deki gıdaların kalori, makro, mineral ve vitamin değerlerini sunan açık ve ücretsiz bir API. 

#### Şu anda geliştirme aşamasındadır ve kullanıma hazır değildir. API aktif olduğunda burası güncellenecektir.

## Gereksinimler
- [Git](https://git-scm.com/downloads)
- [Rust Programlama Dili ve Cargo](https://www.rust-lang.org/tools/install)

## Kurulum
1. Öncelikle kaynak kodunu bilgisayarınıza indirin: `git clone https://github.com/karahanbuhan/besinveri.git`
2. Daha sonrasında kaynak kodunun olduğu ana klasöre girin: `cd besinveri`
3. En hızlı şekilde çalıştırmak için `cargo run` komutunu kullanabilirsiniz.

## Docker ile Kurulum
1. Öncelikle kaynak kodunu bilgisayarınıza indirin: `git clone https://github.com/karahanbuhan/besinveri.git`
2. Daha sonrasında kaynak kodunun olduğu ana klasöre girin: `cd besinveri`
3. Docker Image oluşturmak ve bunu konteyner olarak çalıştırmak için: `docker build --tag besinveri . && docker run -p 8099:8099 --detach besinveri`

## Kurulum Sonrası Kılavuz
Kurulum yaptıktan ve BesinVeri'yi çalıştırdıktan sonra, :8099/TCP portundan API ve siteye bağlanabilirsiniz. Eğer localhost üzerinden açtıysanız, http://localhost:8099/ adresine girerek kurulumun başarılı olduğuna emin olun. Eğer hata alıyorsanız, kurulumu doğru yaptığınıza ve BesinVeri'nin açık olduğuna emin olun.

***Dikkat:** Docker ile çalıştırmak, `cargo run --release` kullanıldığı ve Image inşaa edildiği için birkaç dakika sürebilir. Bu sebepten dolayı, sadece arayüzün son kullanıcıya sunulduğu ortamda bu şekilde çalıştırılması önerilir.*

