"use client";

/**
 * Page de jeu R2
 *
 * Interface principale du MMO spatial avec Canvas 2D.
 */

import { useEffect, useRef, useState } from 'react';
import { useAuth } from '@/contexts/auth';
import { useRouter } from 'next/navigation';

interface Entity {
  id: string;
  x: number;
  y: number;
  type: 'player' | 'ship' | 'asteroid' | 'station';
  rotation?: number;
}

export default function GamePage() {
  const { auth } = useAuth();
  const router = useRouter();
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Utiliser useRef pour les valeurs qui ne doivent pas déclencher de re-render
  const entitiesRef = useRef<Entity[]>([]);
  const playerPosRef = useRef({ x: 400, y: 300 });
  const targetPosRef = useRef<{ x: number; y: number } | null>(null);

  // State uniquement pour l'UI (pas utilisé dans le loop de rendu)
  const [, forceUpdate] = useState({});

  // Rediriger si pas connecté (après initialisation)
  useEffect(() => {
    // Attendre que l'auth soit initialisée avant de décider
    if (!auth.isInitialized) return;

    if (!auth.isAuthenticated) {
      router.push('/');
    }
  }, [auth.isAuthenticated, auth.isInitialized, router]);

  // Initialiser le canvas (une seule fois)
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Redimensionner le canvas
    const resizeCanvas = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);

    // Créer quelques entités de démo
    entitiesRef.current = [
      { id: '1', x: 200, y: 150, type: 'asteroid' },
      { id: '2', x: 600, y: 200, type: 'asteroid' },
      { id: '3', x: 500, y: 450, type: 'asteroid' },
      { id: '4', x: 300, y: 400, type: 'station' },
      { id: '5', x: 700, y: 350, type: 'ship' },
    ];

    // Boucle de rendu
    let animationId: number;
    const render = () => {
      // Fond spatial
      ctx.fillStyle = '#0a0a0f';
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      // Étoiles de fond
      ctx.fillStyle = '#ffffff';
      for (let i = 0; i < 100; i++) {
        const x = (i * 137.5) % canvas.width;
        const y = (i * 89.3) % canvas.height;
        ctx.globalAlpha = 0.3 + Math.random() * 0.7;
        ctx.fillRect(x, y, 1, 1);
      }
      ctx.globalAlpha = 1;

      // Grille spatiale
      ctx.strokeStyle = '#1a1a2e';
      ctx.lineWidth = 1;
      const gridSize = 50;
      for (let x = 0; x < canvas.width; x += gridSize) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, canvas.height);
        ctx.stroke();
      }
      for (let y = 0; y < canvas.height; y += gridSize) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(canvas.width, y);
        ctx.stroke();
      }

      // Dessiner les entités
      entitiesRef.current.forEach(entity => {
        ctx.save();
        ctx.translate(entity.x, entity.y);

        switch (entity.type) {
          case 'player':
            // Vaisseau du joueur (triangle bleu)
            ctx.fillStyle = '#3b82f6';
            ctx.beginPath();
            ctx.moveTo(0, -15);
            ctx.lineTo(-10, 10);
            ctx.lineTo(10, 10);
            ctx.closePath();
            ctx.fill();
            // Moteur
            ctx.fillStyle = '#f59e0b';
            ctx.beginPath();
            ctx.arc(0, 12, 5, 0, Math.PI * 2);
            ctx.fill();
            break;

          case 'ship':
            // Autre vaisseau (triangle gris)
            ctx.fillStyle = '#6b7280';
            ctx.beginPath();
            ctx.moveTo(0, -12);
            ctx.lineTo(-8, 8);
            ctx.lineTo(8, 8);
            ctx.closePath();
            ctx.fill();
            break;

          case 'asteroid':
            // Astéroïde (forme irrégulière marron)
            ctx.fillStyle = '#78716c';
            ctx.beginPath();
            ctx.moveTo(-15, -10);
            ctx.lineTo(-5, -18);
            ctx.lineTo(8, -15);
            ctx.lineTo(18, -5);
            ctx.lineTo(20, 8);
            ctx.lineTo(12, 18);
            ctx.lineTo(-5, 20);
            ctx.lineTo(-18, 12);
            ctx.closePath();
            ctx.fill();
            // Cratères
            ctx.fillStyle = '#57534e';
            ctx.beginPath();
            ctx.arc(-5, 0, 4, 0, Math.PI * 2);
            ctx.fill();
            ctx.beginPath();
            ctx.arc(8, 5, 3, 0, Math.PI * 2);
            ctx.fill();
            break;

          case 'station':
            // Station spatiale (carré vert avec détails)
            ctx.fillStyle = '#22c55e';
            ctx.fillRect(-20, -20, 40, 40);
            // Lights
            ctx.fillStyle = '#86efac';
            ctx.fillRect(-15, -15, 8, 8);
            ctx.fillRect(7, -15, 8, 8);
            ctx.fillRect(-15, 7, 8, 8);
            ctx.fillRect(7, 7, 8, 8);
            // Antenne
            ctx.strokeStyle = '#22c55e';
            ctx.lineWidth = 3;
            ctx.beginPath();
            ctx.moveTo(0, -20);
            ctx.lineTo(0, -30);
            ctx.stroke();
            ctx.fillStyle = '#ef4444';
            ctx.beginPath();
            ctx.arc(0, -30, 3, 0, Math.PI * 2);
            ctx.fill();
            break;
        }

        ctx.restore();
      });

      // Dessiner le joueur
      const playerPos = playerPosRef.current;
      ctx.save();
      ctx.translate(playerPos.x, playerPos.y);

      // Corps du vaisseau
      ctx.fillStyle = '#3b82f6';
      ctx.beginPath();
      ctx.moveTo(0, -20);
      ctx.lineTo(-15, 15);
      ctx.lineTo(0, 10);
      ctx.lineTo(15, 15);
      ctx.closePath();
      ctx.fill();

      // Cockpit
      ctx.fillStyle = '#60a5fa';
      ctx.beginPath();
      ctx.ellipse(0, -5, 5, 8, 0, 0, Math.PI * 2);
      ctx.fill();

      // Moteurs
      ctx.fillStyle = '#f59e0b';
      ctx.globalAlpha = 0.8 + Math.random() * 0.2;
      ctx.beginPath();
      ctx.arc(-10, 18, 4, 0, Math.PI * 2);
      ctx.fill();
      ctx.beginPath();
      ctx.arc(10, 18, 4, 0, Math.PI * 2);
      ctx.fill();
      ctx.globalAlpha = 1;

      ctx.restore();

      // Indicateur de cible
      const targetPos = targetPosRef.current;
      if (targetPos) {
        ctx.strokeStyle = '#22c55e';
        ctx.lineWidth = 2;
        ctx.setLineDash([5, 5]);
        ctx.beginPath();
        ctx.arc(targetPos.x, targetPos.y, 20, 0, Math.PI * 2);
        ctx.stroke();
        ctx.setLineDash([]);

        // Ligne vers la cible
        ctx.strokeStyle = 'rgba(34, 197, 94, 0.3)';
        ctx.beginPath();
        ctx.moveTo(playerPos.x, playerPos.y);
        ctx.lineTo(targetPos.x, targetPos.y);
        ctx.stroke();
      }

      animationId = requestAnimationFrame(render);
    };

    render();

    return () => {
      window.removeEventListener('resize', resizeCanvas);
      cancelAnimationFrame(animationId);
    };
  }, [auth.isInitialized]); // S'exécute quand le canvas est dans le DOM

  // Clic pour définir la cible de mouvement
  const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    targetPosRef.current = { x, y };

    // Simuler le mouvement vers la cible
    const startX = playerPosRef.current.x;
    const startY = playerPosRef.current.y;
    const dx = x - startX;
    const dy = y - startY;
    const distance = Math.sqrt(dx * dx + dy * dy);
    const speed = 3;

    if (distance > speed) {
      const steps = Math.ceil(distance / speed);
      let step = 0;

      const move = setInterval(() => {
        step++;
        const progress = step / steps;
        playerPosRef.current = {
          x: startX + dx * progress,
          y: startY + dy * progress,
        };

        // Forcer le re-render pour mettre à jour l'UI
        forceUpdate({});

        if (step >= steps) {
          clearInterval(move);
          targetPosRef.current = null;
        }
      }, 16);
    } else {
      targetPosRef.current = null;
    }
  };

  // Afficher l'état de chargement pendant l'initialisation
  if (!auth.isInitialized) {
    return (
      <div className="min-h-screen bg-black flex items-center justify-center">
        <p className="text-gray-400">Chargement...</p>
      </div>
    );
  }

  // Rediriger si pas connecté
  if (!auth.isAuthenticated) {
    return null;  // Le useEffect va rediriger
  }

  const playerPos = playerPosRef.current;

  return (
    <div className="relative w-full h-screen overflow-hidden">
      {/* Canvas de jeu */}
      <canvas
        ref={canvasRef}
        onClick={handleCanvasClick}
        className="block cursor-crosshair"
      />

      {/* UI Overlay */}
      <div className="absolute top-0 left-0 right-0 p-4 flex justify-between items-start pointer-events-none">
        {/* Info joueur */}
        <div className="bg-gray-900/80 backdrop-blur-sm rounded-lg p-3 border border-gray-700 pointer-events-auto">
          <p className="text-white font-semibold">Joueur #{auth.playerId}</p>
          <p className="text-gray-400 text-sm">Position: ({Math.round(playerPos.x)}, {Math.round(playerPos.y)})</p>
        </div>

        {/* Mini stats */}
        <div className="bg-gray-900/80 backdrop-blur-sm rounded-lg p-3 border border-gray-700 pointer-events-auto flex gap-4">
          <div>
            <p className="text-xs text-gray-500">Hull</p>
            <div className="w-24 h-2 bg-gray-700 rounded-full overflow-hidden">
              <div className="h-full bg-green-500 w-[85%]"></div>
            </div>
          </div>
          <div>
            <p className="text-xs text-gray-500">Energy</p>
            <div className="w-24 h-2 bg-gray-700 rounded-full overflow-hidden">
              <div className="h-full bg-blue-500 w-[60%]"></div>
            </div>
          </div>
        </div>
      </div>

      {/* Hint */}
      <div className="absolute bottom-4 left-1/2 -translate-x-1/2 bg-gray-900/80 backdrop-blur-sm rounded-lg px-4 py-2 border border-gray-700 pointer-events-none">
        <p className="text-gray-400 text-sm">Clic pour déplacer • Esc pour quitter</p>
      </div>

      {/* Bouton déconnexion */}
      <button
        onClick={() => {
          localStorage.removeItem('r2_auth');
          window.location.href = '/';
        }}
        className="absolute bottom-4 right-4 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm transition-colors"
      >
        Déconnexion
      </button>
    </div>
  );
}
