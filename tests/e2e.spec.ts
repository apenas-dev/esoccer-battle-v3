import { test, expect } from '@playwright/test';

test.describe('E-Soccer Battle V3', () => {

  // ── Teste 1 — Renderização da MatchPage ─────────────
  test('renderização inicial da MatchPage', async ({ page }) => {
    await page.goto('/');

    await expect(page.locator('text=E-Soccer')).toBeVisible();
    await expect(page.locator('text=Time A')).toBeVisible();
    await expect(page.locator('text=Time B')).toBeVisible();
    await expect(page.getByRole('img', { name: /Placar time A: 0 gols/ })).toBeVisible();
    await expect(page.getByRole('img', { name: /Placar time B: 0 gols/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Iniciar/ })).toBeVisible();
  });

  // ── Teste 2 — Fluxo completo de uma partida ────────
  test('fluxo completo de uma partida', async ({ page }) => {
    await page.goto('/');

    // Iniciar
    await page.getByRole('button', { name: /Iniciar/ }).click();

    // Timer inicia
    await page.waitForTimeout(1500);
    const timerText = await page.locator('section[aria-label="Cronômetro"]').textContent();
    expect(timerText).not.toBe('00:00');

    // Voice ativo
    const voiceSection = page.locator('section[aria-label="Indicador de voz"]');
    await expect(voiceSection).toBeVisible();

    // Gol A → 1x0
    await page.getByRole('button', { name: /Gol para Time A/ }).click();
    await expect(page.getByRole('img', { name: /Placar time A: 1 gols/ })).toBeVisible();
    await expect(page.getByRole('img', { name: /Placar time B: 0 gols/ })).toBeVisible();

    // Gol B → 1x1
    await page.getByRole('button', { name: /Gol para Time B/ }).click();
    await expect(page.getByRole('img', { name: /Placar time A: 1 gols/ })).toBeVisible();
    await expect(page.getByRole('img', { name: /Placar time B: 1 gols/ })).toBeVisible();

    // Dúvida → challenge
    await page.getByRole('button', { name: /Dúvida/ }).click();
    await expect(page.getByRole('status', { name: /DÚVIDA/ })).toBeVisible();

    // Volta Seis
    await page.getByRole('button', { name: /Desfazer último comando/ }).click();
    const cmdLog = page.locator('section[aria-label="Log de comandos"]');
    await expect(cmdLog.locator('text=Volta seis')).toBeVisible();

    // Encerrar
    await page.getByRole('button', { name: /Encerrar/ }).click();
    await expect(page.getByRole('status', { name: /ENCERRADO/ })).toBeVisible();
  });

  // ── Teste 3 — Controles contextuais ────────────────
  test('controles contextuais mudam conforme status', async ({ page }) => {
    await page.goto('/');

    // Idle: só Iniciar
    await expect(page.getByRole('button', { name: /Iniciar/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Gol para Time A/ })).not.toBeVisible();
    await expect(page.getByRole('button', { name: /Encerrar/ })).not.toBeVisible();

    // Jogando
    await page.getByRole('button', { name: /Iniciar/ }).click();
    await expect(page.getByRole('button', { name: /Gol para Time A/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Gol para Time B/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Dúvida/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Desfazer/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Encerrar/ })).toBeVisible();

    // Encerrado
    await page.getByRole('button', { name: /Encerrar/ }).click();
    await expect(page.getByRole('button', { name: /Nova Partida|Iniciar/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Gol para Time A/ })).not.toBeVisible();
  });

  // ── Teste 4 — Responsividade ───────────────────────
  test('layout responsivo desktop', async ({ browser }) => {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await ctx.newPage();
    await page.goto('/');
    await expect(page.locator('text=E-Soccer')).toBeVisible();
    await expect(page.locator('text=Time A')).toBeVisible();
    await expect(page.locator('text=Time B')).toBeVisible();
    await ctx.close();
  });

  test('layout responsivo mobile', async ({ browser }) => {
    const ctx = await browser.newContext({ viewport: { width: 375, height: 667 } });
    const page = await ctx.newPage();
    await page.goto('/');
    await expect(page.locator('text=E-Soccer')).toBeVisible();
    await expect(page.locator('text=Time A')).toBeVisible();
    await expect(page.locator('text=Time B')).toBeVisible();
    await ctx.close();
  });
});
