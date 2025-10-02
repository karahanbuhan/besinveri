import asyncio
import httpx
import time
from datetime import datetime

# 150 farklı sorgu, 28 karakter (96 bayt altında)
BASE_URL = "http://localhost:8099/api/foods/search"
QUERIES = [f"q={'A' * 28}_{i:03d}" for i in range(150)]  # q=AAA..._000, q=AAA..._001, ...
URLS = [f"{BASE_URL}?{query}" for query in QUERIES]

# Rate limit: Saniyede 4 istek
RATE_LIMIT = 4
COOLDOWN = 1.0  # 1 saniye cooldown

async def send_get_request(url: str, client: httpx.AsyncClient, semaphore: asyncio.Semaphore) -> dict:
    """Belirtilen URL'ye GET isteği gönderir ve süreyi ölçer."""
    async with semaphore:
        start_time = time.time()
        try:
            response = await client.get(url, timeout=5.0)
            elapsed_time = (time.time() - start_time) * 1000  # Milisaniye
            await asyncio.sleep(COOLDOWN)
            return {
                "url": url,
                "status": response.status_code,
                "duration_ms": round(elapsed_time, 2),
                "success": response.status_code == 200,
                "response": response.text[:100] if response.status_code == 200 else f"Hata: {response.status_code}",
            }
        except httpx.RequestError as e:
            elapsed_time = (time.time() - start_time) * 1000
            await asyncio.sleep(COOLDOWN)
            return {
                "url": url,
                "status": None,
                "duration_ms": round(elapsed_time, 2),
                "success": False,
                "response": f"Hata: {str(e)}",
            }

async def stress_test():
    """150 farklı sorguya istek gönderir, rate limit ile."""
    print(f"Test başlıyor: {datetime.now().isoformat()}")
    semaphore = asyncio.Semaphore(RATE_LIMIT)

    async with httpx.AsyncClient() as client:
        # 150 isteği 10'arlı gruplar halinde gönder
        for i in range(0, len(URLS), 10):
            batch_urls = URLS[i:i+10]
            tasks = [send_get_request(url, client, semaphore) for url in batch_urls]
            results = await asyncio.gather(*tasks, return_exceptions=True)

            # Sonuçları yazdır
            print(f"\nGrup {i//10 + 1} Sonuçları:")
            print("-" * 50)
            for result in results:
                print(f"URL: {result['url']}")
                print(f"Durum: {'Başarılı' if result['success'] else 'Başarısız'}")
                print(f"HTTP Kodu: {result['status'] or 'Yok'}")
                print(f"Süre: {result['duration_ms']} ms")
                print(f"Yanıt: {result['response']}")
                print("-" * 50)

if __name__ == "__main__":
    asyncio.run(stress_test())