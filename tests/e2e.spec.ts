import { test, expect } from '@playwright/test';

test.describe('E-Soccer Battle V3', () => {

  // ── Teste 1 — Renderização da MatchPage ─────────────
  test('renderização inicial da MatchPage', async ({ page }) => {
    await page.goto('/');

    // Header — "E-Soccer" aparece no header e no footer, usar exact
    await expect(page.getByText('E-Soccer', { exact: true })).toBeVisible();

    // Times — usar heading pra evitar match com footer
    await expect(page.getByRole('heading', { name: 'Time A' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Time B' })).toBeVisible();

    // Placar
    await expect(page.getByRole('img', { name: /Placar time A/ })).toBeVisible();
    await expect(page.getByRole('img', { name: /Placar time B/ })).toBeVisible();

    // Status AGUARDANDO
    await expect(page.getByRole('status', { name: /Status/ })).toContainText('AGUARDANDO');

    // Botão Iniciar
    await expect(page.getByRole('button', { name: /Iniciar/ })).toBeVisible();
  });

  // ── Teste 2 — Fluxo completo de uma partida ────────
  test('fluxo completo de uma partida', async ({ page }) => {
    await page.goto('/');

    // Iniciar partida
    await page.getByRole('button', { name: /Iniciar/ }).click();

    // Timer inicia
    await page.waitForTimeout(2000);
    const timerText = await page.locator('section[aria-label="Cronômetro"]').textContent();
    expect(timerText).not.toBe('00:00');

    // Voice indicator visível
    await expect(page.locator('section[aria-label="Indicador de voz"]')).toBeVisible();

    // Gol A
    await page.getByRole('button', { name: /Gol.*Time A/ }).first().click();

    // Gol B
    await page.getByRole('button', { name: /Gol.*Time B/ }).first().click();

    // Verificar placar visível
    const scoreboard = page.getByRole('region', { name: 'Placar da partida' });
    await expect(scoreboard).toBeVisible();

    // Dúvida (challenge)
    const challengeBtn = page.getByRole('button', { name: /Dúvida/ });
    if (await challengeBtn.isVisible()) {
      await challengeBtn.click();
    }

    // Encerrar
    await page.getByRole('button', { name: /Encerrar/ }).click();

    // Status final — usar aria-label específico pra evitar ambiguidade com voice status
    await expect(page.getByRole('status', { name: /Status/ })).toContainText('ENCERRADO');
  });

  // ── Teste 3 — Controles contextuais ────────────────
  test('controles contextuais mudam conforme status', async ({ page }) => {
    await page.goto('/');

    // Idle: Iniciar visível
    await expect(page.getByRole('button', { name: /Iniciar/ })).toBeVisible();

    // Jogando
    await page.getByRole('button', { name: /Iniciar/ }).click();
    await page.waitForTimeout(500);

    // Após iniciar: encerrar deve estar disponível
    const endBtn = page.getByRole('button', { name: /Encerrar/ });
    await expect(endBtn).toBeVisible();

    // Encerrar
    await endBtn.click();

    // Após encerrar: Iniciar deve aparecer novamente
    await expect(page.getByRole('button', { name: /Iniciar/ })).toBeVisible();
  });

  // ── Teste 4 — Responsividade desktop ───────────────
  test('layout responsivo desktop', async ({ browser }) => {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await ctx.newPage();
    await page.goto('/');
    await expect(page.getByText('E-Soccer', { exact: true })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Time A' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Time B' })).toBeVisible();
    await expect(page.getByRole('button', { name: /Iniciar/ })).toBeVisible();
    await ctx.close();
  });

  // ── Teste 5 — Responsividade mobile ────────────────
  test('layout responsivo mobile', async ({ browser }) => {
    const ctx = await browser.newContext({ viewport: { width: 375, height: 667 } });
    const page = await ctx.newPage();
    await page.goto('/');
    await expect(page.getByText('E-Soccer', { exact: true })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Time A' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Time B' })).toBeVisible();
    await expect(page.getByRole('button', { name: /Iniciar/ })).toBeVisible();
    await ctx.close();
  });
});
