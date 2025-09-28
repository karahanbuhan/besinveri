import asyncio
import httpx
import time
from datetime import datetime

# Test edilecek URL'ler
URLS = [
    "http://localhost:8099/api/health",
    "http://localhost:8099/api/foods/list",
]


async def send_get_request(url: str, client: httpx.AsyncClient) -> dict:
    """Belirtilen URL'ye GET isteği gönderir ve süreyi ölçer."""
    start_time = time.time()
    try:
        response = await client.get(url, timeout=5.0)  # 5 saniye timeout
        elapsed_time = (time.time() - start_time) * 1000  # Milisaniye
        return {
            "url": url,
            "status": response.status_code,
            "duration_ms": round(elapsed_time, 2),
            "success": response.status_code == 200,
            "response": response.text[:100],  # İlk 100 karakter
        }
    except httpx.RequestError as e:
        elapsed_time = (time.time() - start_time) * 1000
        return {
            "url": url,
            "status": None,
            "duration_ms": round(elapsed_time, 2),
            "success": False,
            "response": f"Hata: {str(e)}",
        }


async def test_endpoints():
    """Paralel olarak tüm URL'lere istek gönderir."""
    print(f"Test başlıyor: {datetime.now().isoformat()}")
    async with httpx.AsyncClient() as client:
        tasks = [send_get_request(url, client) for url in URLS]
        results = await asyncio.gather(*tasks, return_exceptions=True)

    print("\nSonuçlar:")
    print("-" * 50)
    for result in results:
        print(f"URL: {result['url']}")
        print(f"Durum: {'Başarılı' if result['success'] else 'Başarısız'}")
        print(f"HTTP Kodu: {result['status'] or 'Yok'}")
        print(f"Süre: {result['duration_ms']} ms")
        print(f"Yanıt: {result['response']}")
        print("-" * 50)


if __name__ == "__main__":
    asyncio.run(test_endpoints())
