# Elevator Optimization

Boğaziçi AI UpForAChallange adlı yarışmaya katılmak için bu projeyi hazırladık. Bu readme dosyası paper niteliğinde *değildir*.
Paper ayrı bir dosyada.

Ben tuna boğaziçinde inşaat okuyom 1. sınıfım. Diğer arkadaş da yiğit o da 10. sınıf büyüyünce boğaziçine çekicem onu da.
Kodları ciddi bi şekilde yazmaya 23 ekimde başladık o yüzden planladığımız birçok özelliği ekleyemedik ama yapacak bi şey yok.
Yarışma geçtikten sonra kodları düzenlicem dağınık taraflar ve geliştirilmemiş özellikler sinirimi bozuyor.

Cidden asansör algoritmalarının test edilmesi için hoş bi ortam sunmak güzel olurdu.
Temel özellikleri ekleyip kodu düzenledikten sonra kesinlikle async yapıcam pid loopları için çok tatlı olabilir

Yiğit hayatında ilk defa rust kullanıyor ve şu anda yaptığı en büyük yazılım projesi de muhtemelen bu, ben de genelde tek başıma çalışıyorum o yüzden senkronizasyon konusunda biraz sıkıntı yaşadık ve repodaki katkılarımız muhtemelen rastgele gözükücek.
Çalıştığımız saatler de genelde farklı oluyordu. her neyse

Eğer repo başkaları tarafından ilgi görmeye başlarsa bu readmeyi daha ciddi bi şekilde yazarım ama kodlarımı kimsenin incelemediğine emin olduğum için şu anlık böyle kalıcak.

### Kodları beğendiyseniz ve bize iş ayarlayabileceğinizi düşünüyorsanız lütfen söyleyin, şirkete ([nfr productsa](https://nfrproducts.com)) cnc router almak için para biriktiriyoruz.
lütfen vereceğiniz işlerin gerçek deadlinenını söylemek yerine daha erken bir tarih söyleyin genel olarak bi yetiştirememe problemimiz var.

Bi de lanet olsun ki [portage](https://wiki.gentoo.org/wiki/Portage) da [lto](https://wiki.gentoo.org/wiki/LTO) açık, pytorchun rust bindingleri derlenmiyor. eğer bikaç saat içinde sıkıntıyı çözüp derleyebilirsem yapay zeka algoritmasını da yazıcam.
-(bikaç saat sonraki tuna) derlenmedi

## Yapay Zeka konusunda
normalde pytorch bindingleri ile rs torch kullancaktım ama derlerken sıkıntılar çıktı bilgisayarımda yer doldu falan ben de neural ağı baştan yazmaya karar verdim muhtemelen kurulum yapmaktan daha hızlı olucak. 

İleriki versiyonlarda tüm algoritmaların statespace alması en mantıklısı olur

### bu hayat gerçekten insanozor
gece gündüz yazılım yapmayı özlemişim, eğlenceli birkaç günün de sonuna geldik :-(
