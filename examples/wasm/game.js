// Configuration constants
const CONFIG = {
  GROUND_HEIGHT: 22,
  CANVAS: {
    WIDTH: 600,
    HEIGHT: 300,
  },
  PLAYER: {
    WIDTH: 80,
    HEIGHT: 40,
    START_X: 50,
    JUMP_VELOCITY: -18,
  },
  PHYSICS: {
    GRAVITY: 0.8,
  },
  OBSTACLE: {
    WIDTH: 15,
    HEIGHT: 30,
    SPAWN_INTERVAL: 90,
    MIN_GAP: 200,
  },
  GAME: {
    INITIAL_SPEED: 3,
    TARGET_FPS: 60,
    FONT_FACE: "Press Start 2P",
  },
  TRAIL: {
    BASE_WIDTH: 8,
    JUMP_WIDTH_BONUS: 6,
    SPAWN_INTERVAL: 2,
    PARTICLE_LIFETIME: 40,
  },
};

const COLORS = {
  RED: "#FF4C4C",
  ORANGE: "#FF7F50",
  YELLOW: "#FFD93D",
  GREEN: "#32D74B",
  BLUE: "#5AC8FA",
  PURPLE: "#AF52DE",
  GRAY: "#4B5563",
  WHITE: "#FFFFFF",
  BLACK: "#0B0B0B",
};

const OBSTACLE_TYPES = [
  { color: COLORS.RED, heightMultiplier: 3 },
  { color: COLORS.BLUE, heightMultiplier: 1 },
  { color: COLORS.YELLOW, heightMultiplier: 0.5 },
  { color: COLORS.GREEN, heightMultiplier: 0 },
];

// Utility functions
const Utils = {
  clamp: (value, min, max) => Math.max(min, Math.min(max, value)),
  random: (min, max) => Math.random() * (max - min) + min,
  hexToRgba: (hex, alpha) => {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return `rgba(${r},${g},${b},${alpha.toFixed(2)})`;
  },
  checkCollision: (rect1, rect2) => {
    return (
      rect1.x < rect2.x + rect2.width &&
      rect1.x + rect1.width > rect2.x &&
      rect1.y < rect2.y + rect2.height &&
      rect1.y + rect1.height > rect2.y
    );
  },
};

// Game elements
const canvas = document.getElementById("game-canvas");
canvas.width = CONFIG.CANVAS.WIDTH;
canvas.height = CONFIG.CANVAS.HEIGHT;

const ctx = canvas.getContext("2d");
const UI = {
  gameOverMessage: document.getElementById("game-over"),
  restartButton: document.getElementById("restart-button"),
  shareButton: document.getElementById("share-button"),
  floatingText: "keymap-rs",
};

// Calculated constants
const GROUND_Y = canvas.height - CONFIG.GROUND_HEIGHT;

// Image loading
const playerImage = new Image();
playerImage.src = "public/nyan_cat.gif";

class Player {
  constructor() {
    this.reset();
  }

  reset() {
    this.x = CONFIG.PLAYER.START_X;
    this.y = GROUND_Y - CONFIG.PLAYER.HEIGHT;
    this.width = CONFIG.PLAYER.WIDTH;
    this.height = CONFIG.PLAYER.HEIGHT;
    this.velocityY = 0;
    this.isJumping = false;
    this.direction = 1;
    this.movement = { left: false, right: false };
  }

  jump() {
    if (!this.isJumping) {
      this.isJumping = true;
      this.velocityY = CONFIG.PLAYER.JUMP_VELOCITY;
    }
  }

  setMovement(direction, isMoving) {
    this.movement[direction] = isMoving;
    if (isMoving) {
      this.direction = direction === "left" ? -1 : 1;
    }
  }

  update(gameSpeed) {
    this._updateJump();
    this._updateHorizontalMovement(gameSpeed);
  }

  _updateJump() {
    if (this.isJumping) {
      this.y += this.velocityY;
      this.velocityY += CONFIG.PHYSICS.GRAVITY;

      if (this.y >= GROUND_Y - this.height) {
        this.y = GROUND_Y - this.height;
        this.isJumping = false;
        this.velocityY = 0;
      }
    }
  }

  _updateHorizontalMovement(gameSpeed) {
    if (this.movement.left) {
      this.x = Math.max(0, this.x - gameSpeed);
    }
    if (this.movement.right) {
      this.x = Math.min(canvas.width - this.width, this.x + gameSpeed);
    }
  }

  draw() {
    if (!playerImage.complete) return;

    ctx.save();
    if (this.direction === -1) {
      ctx.translate(this.x + this.width, this.y);
      ctx.scale(-1, 1);
      ctx.drawImage(playerImage, 0, 0, this.width, this.height);
    } else {
      ctx.drawImage(playerImage, this.x, this.y, this.width, this.height);
    }
    ctx.restore();
  }

  getBounds() {
    return {
      x: this.x,
      y: this.y,
      width: this.width,
      height: this.height,
    };
  }
}

class ParallaxBackground {
  constructor() {
    this.layers = [
      { speed: 0.2, size: 1, count: 100 },
      { speed: 0.5, size: 2, count: 50 },
      { speed: 1.0, size: 3, count: 25 },
    ].map((layer) => ({ ...layer, stars: [] }));

    this._initializeLayers();
  }

  _initializeLayers() {
    this.layers.forEach((layer) => {
      for (let i = 0; i < layer.count; i++) {
        layer.stars.push({
          x: Utils.random(0, canvas.width),
          y: Utils.random(0, GROUND_Y - 20),
          size: layer.size,
        });
      }
    });
  }

  update(gameSpeed) {
    this.layers.forEach((layer) => {
      layer.stars.forEach((star) => {
        star.x -= layer.speed * gameSpeed;
        if (star.x < 0) {
          star.x = canvas.width;
          star.y = Utils.random(0, GROUND_Y - 20);
        }
      });
    });
  }

  draw() {
    ctx.fillStyle = COLORS.WHITE;
    this.layers.forEach((layer) => {
      layer.stars.forEach((star) => {
        ctx.beginPath();
        ctx.arc(star.x, star.y, star.size / 2, 0, Math.PI * 2);
        ctx.fill();
      });
    });
  }
}

class FloatingText {
  constructor(text, x, y, speed) {
    this.text = text;
    this.x = x;
    this.y = y;
    this.speed = speed;
    this.hue = 0;
  }

  update() {
    this.x -= this.speed;
    if (this.x < -ctx.measureText(this.text).width) {
      this.x = canvas.width;
      this.y = Utils.random(0, GROUND_Y - 50);
    }
    this.hue = (this.hue + 1) % 360;
  }

  draw() {
    const lightness = 50 + 20 * Math.sin((this.hue / 360) * Math.PI);

    ctx.save();
    ctx.font = `20px "${CONFIG.GAME.FONT_FACE}"`;
    ctx.fillStyle = `hsl(${this.hue}, 100%, ${lightness}%)`;
    ctx.fillText(this.text, this.x, this.y);
    ctx.restore();
  }
}

class ObstacleManager {
  constructor(gameSpeed) {
    this.gameSpeed = gameSpeed;
    this.obstacles = [];
    this.frameCount = 0;
  }

  reset() {
    this.obstacles = [];
    this.frameCount = 0;
  }

  update() {
    this.frameCount++;
    this._spawnObstacles();
    this._updateObstacles();
    this._removeOffscreenObstacles();
  }

  _spawnObstacles() {
    if (
      this.frameCount % CONFIG.OBSTACLE.SPAWN_INTERVAL === 0 ||
      this.obstacles.length === 0
    ) {
      const height = CONFIG.OBSTACLE.HEIGHT + Math.random() * 50;
      const lastObstacle = this.obstacles[this.obstacles.length - 1];
      const randomOffset = Math.random() * 100;

      const spawnX = lastObstacle
        ? Math.max(
          canvas.width,
          lastObstacle.x + CONFIG.OBSTACLE.MIN_GAP + randomOffset,
        )
        : canvas.width + randomOffset;

      const type =
        OBSTACLE_TYPES[Math.floor(Math.random() * OBSTACLE_TYPES.length)];
      const dHeight =
        (Math.random() > 0.5 ? 1 : -1) * 0.5 * type.heightMultiplier;

      this.obstacles.push({
        x: spawnX,
        y: GROUND_Y - height,
        width: CONFIG.OBSTACLE.WIDTH,
        height,
        minHeight: CONFIG.OBSTACLE.HEIGHT,
        maxHeight: CONFIG.OBSTACLE.HEIGHT + 50,
        dHeight,
        scored: false,
        counted: false,
        type,
        color: type.color,
      });
    }
  }

  _updateObstacles() {
    this.obstacles.forEach((obstacle) => {
      obstacle.x -= this.gameSpeed;
      obstacle.height += obstacle.dHeight;
      obstacle.y = GROUND_Y - obstacle.height;

      if (
        obstacle.height > obstacle.maxHeight ||
        obstacle.height < obstacle.minHeight
      ) {
        obstacle.dHeight *= -1;
      }
    });
  }

  _removeOffscreenObstacles() {
    this.obstacles = this.obstacles.filter(
      (obstacle) => obstacle.x + obstacle.width >= 0,
    );
  }

  checkCollisions(player) {
    const playerBounds = player.getBounds();
    return this.obstacles.some((obstacle) =>
      Utils.checkCollision(playerBounds, obstacle),
    );
  }

  updateScoring(player, onScore) {
    this.obstacles.forEach((obstacle) => {
      if (!obstacle.scored && obstacle.x + obstacle.width < player.x) {
        obstacle.scored = true;
        if (!obstacle.counted) {
          obstacle.counted = true;
          onScore();
        }
      }
    });
  }

  draw() {
    this.obstacles.forEach((obstacle) => {
      ctx.fillStyle = obstacle.color;
      ctx.fillRect(obstacle.x, obstacle.y, obstacle.width, obstacle.height);
    });
  }
}

class RainbowTrail {
  constructor(player) {
    this.player = player;
    this.colors = [
      COLORS.RED,
      COLORS.ORANGE,
      COLORS.YELLOW,
      COLORS.GREEN,
      COLORS.BLUE,
      COLORS.PURPLE,
    ];
    this.reset();
  }

  reset() {
    this.particles = [];
    this.frameCount = 0;
  }

  update() {
    this.frameCount++;
    this._spawnParticles();
    this._updateParticles();
    this._removeDeadParticles();
  }

  _spawnParticles() {
    if (this.frameCount % CONFIG.TRAIL.SPAWN_INTERVAL === 0) {
      const particleWidth =
        CONFIG.TRAIL.BASE_WIDTH +
        (this.player.isJumping ? CONFIG.TRAIL.JUMP_WIDTH_BONUS : 0);

      this.colors.forEach((color, i) => {
        this.particles.push({
          x: this.player.x + 10,
          y: this.player.y + 5 + i * 5,
          color,
          alpha: 1.0,
          lifetime: CONFIG.TRAIL.PARTICLE_LIFETIME,
          width: particleWidth,
          height: 4,
        });
      });
    }
  }

  _updateParticles() {
    this.particles.forEach((particle) => {
      particle.x -= 1.5;
      particle.alpha -= 1 / particle.lifetime;
    });
  }

  _removeDeadParticles() {
    this.particles = this.particles.filter((particle) => particle.alpha > 0);
  }

  draw() {
    this.particles.forEach((particle) => {
      ctx.save();
      ctx.globalAlpha = particle.alpha;
      ctx.fillStyle = particle.color;
      ctx.fillRect(particle.x, particle.y, particle.width, particle.height);
      ctx.restore();
    });
  }
}

class FPSCounter {
  constructor() {
    this.fps = 0;
    this.lastUpdate = performance.now();
    this.frameCounter = 0;

    // Add fixed timestep for consistent FPS calculation
    this.logicFrameCounter = 0;
    this.logicFPS = 0;
  }

  update() {
    this.frameCounter++; // This counts render frames (will be 120 on Chrome)
    const now = performance.now();

    if (now - this.lastUpdate >= 500) {
      this.fps = (this.frameCounter * 1000) / (now - this.lastUpdate);
      this.lastUpdate = now;
      this.frameCounter = 0;
    }
  }

  draw() {
    ctx.fillStyle = COLORS.WHITE;
    ctx.font = `12px "${CONFIG.GAME.FONT_FACE}"`;
    ctx.textAlign = "right";
    ctx.fillText(`FPS: ${this.fps.toFixed(2)}`, canvas.width - 10, 20);
    ctx.textAlign = "left";
  }
}

class Game {
  constructor() {
    this.player = new Player();
    this.rainbowTrail = new RainbowTrail(this.player);
    this.obstacleManager = new ObstacleManager(CONFIG.GAME.INITIAL_SPEED);
    this.background = new ParallaxBackground();
    this.floatingText = new FloatingText(
      UI.floatingText,
      canvas.width,
      canvas.height / 2 - 20,
      1.5,
    );
    this.fpsCounter = new FPSCounter();

    this.score = 0;
    this.gameSpeed = CONFIG.GAME.INITIAL_SPEED;
    this.gameOver = false;
    this.paused = false;
    this.key = "";

    // Delta time approach instead of frame limiting
    this.lastTime = 0;
    this.targetFrameTime = 1000 / CONFIG.GAME.TARGET_FPS; // 16.67ms for 60fps
    this.accumulator = 0;
    this.animationFrameId = null;

    this._setupUI();
  }

  _setupUI() {
    // Focus on canvas
    canvas.focus();

    UI.restartButton.addEventListener("click", () => this.reset());
    UI.shareButton.addEventListener("click", () => share(this.score));
  }

  reset() {
    if (this.animationFrameId) {
      cancelAnimationFrame(this.animationFrameId);
    }

    this.player.reset();
    this.obstacleManager.reset();
    this.rainbowTrail.reset();
    this.score = 0;
    this.gameSpeed = CONFIG.GAME.INITIAL_SPEED;
    this.gameOver = false;
    this.paused = false;
    this.key = "";
    this._hideGameOverUI();
    this.accumulator = 0; // Ensure game logic runs on first frame after reset

    // Start a new animation frame loop
    this.animationFrameId = requestAnimationFrame((time) => {
      this.lastTime = time; // Reset lastTime for the new loop
      this.update(time);
    });
  }

  setKey(key, desc) {
    this.key = [key, desc]
      .filter(Boolean)
      .map((s) => s.toLowerCase())
      .join(" - ");
  }

  togglePause() {
    this.paused = !this.paused;
    if (!this.paused) {
      this.lastTime = performance.now();
    }
  }

  _showGameOverUI() {
    UI.gameOverMessage.style.display = "block";
    UI.restartButton.style.display = "block";
    UI.shareButton.style.display = "block";
  }

  _hideGameOverUI() {
    UI.gameOverMessage.style.display = "none";
    UI.restartButton.style.display = "none";
    UI.shareButton.style.display = "none";
  }

  _drawGround() {
    ctx.fillStyle = COLORS.GRAY;
    ctx.fillRect(0, GROUND_Y, canvas.width, CONFIG.GROUND_HEIGHT);
  }

  _drawScore() {
    ctx.fillStyle = COLORS.WHITE;
    ctx.font = `12px "${CONFIG.GAME.FONT_FACE}"`;
    ctx.textAlign = "left";
    ctx.fillText(`Score: ${this.score}`, 10, 20);
  }

  _drawKey() {
    let fontSize = 10;
    ctx.fillStyle = "#ccc";
    ctx.font = `${fontSize}px "${CONFIG.GAME.FONT_FACE}"`;
    ctx.textAlign = "center";
    ctx.fillText(
      this.key,
      canvas.width / 2,
      GROUND_Y + CONFIG.GROUND_HEIGHT - (CONFIG.GROUND_HEIGHT - fontSize) / 2,
    );
  }

  _handleScoring() {
    this.obstacleManager.updateScoring(this.player, () => {
      this.score++;
      this._drawScore();
    });
  }

  _drawPauseOverlay() {
    this._render();
    ctx.save();
    ctx.fillStyle = "rgba(0, 0, 0, 0.5)";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = COLORS.WHITE;
    ctx.font = `40px "${CONFIG.GAME.FONT_FACE}"`;
    ctx.textAlign = "center";
    ctx.fillText("Paused", canvas.width / 2, canvas.height / 2);
    ctx.restore();
    this.animationFrameId = requestAnimationFrame(this.update.bind(this));
  }

  update(currentTime = 0) {
    if (this.gameOver) return;

    if (this.paused) {
      this._drawPauseOverlay();
      return;
    }

    // Delta time calculation
    const deltaTime = currentTime - this.lastTime;
    this.lastTime = currentTime;

    // Accumulate time
    this.accumulator += deltaTime;

    // Fixed timestep updates - ensures consistent game logic
    while (this.accumulator >= this.targetFrameTime) {
      this._updateGameLogic();
      this.accumulator -= this.targetFrameTime;
    }

    // Always render (for smooth visuals)
    this._render();

    this.animationFrameId = requestAnimationFrame(this.update.bind(this));
  }

  _updateGameLogic() {
    // Update game objects at fixed timestep
    this.background.update(this.gameSpeed);
    this.floatingText.update();
    this.player.update(this.gameSpeed);
    this.rainbowTrail.update();

    this.obstacleManager.gameSpeed = this.gameSpeed;
    this.obstacleManager.update();

    this._handleScoring();

    // Check for collisions
    if (this.obstacleManager.checkCollisions(this.player)) {
      this.gameOver = true;
      this._showGameOverUI();
    }

    this.fpsCounter.update();
  }

  _render() {
    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw everything
    this.background.draw();
    this.floatingText.draw();
    this._drawGround();
    this.player.draw();
    this.rainbowTrail.draw();
    this.obstacleManager.draw();
    this._drawScore();
    this._drawKey();

    // Update and draw FPS
    this.fpsCounter.draw();
  }
}

function share(score) {
  const text = `I scored ${score} in Nyan Jump! Try to beat me!`;
  const hashtags = "keymap-rs,rust,wasm";
  const url = `https://x.com/intent/tweet?text=${encodeURIComponent(text)}&url=${encodeURIComponent(window.location)}&hashtags=${hashtags}`;

  window.open(url, "_blank");
}

// Game instance
let game = new Game();

// Exported functions for external control
export function resetGame() {
  game.reset();
}

export function jump() {
  if (!game.gameOver) {
    game.player.jump();
  }
}

export function moveLeft(isMoving) {
  if (!game.gameOver) {
    game.player.setMovement("left", isMoving);
  }
}

export function moveRight(isMoving) {
  if (!game.gameOver) {
    game.player.setMovement("right", isMoving);
  }
}

export function isGameOver() {
  return game.gameOver;
}

export function pauseGame() {
  if (!game.gameOver) {
    game.togglePause();
  }
}

export function setKey(key, description) {
  game.setKey(key, description);
}

export function setSkin(c) {
  // Handle char code or string character
  const char = typeof c === 'number' ? String.fromCharCode(c) : c;
  const digit = parseInt(char);
  if (isNaN(digit)) return;

  // Change rainbow trail colors based on digit
  const baseHue = (digit * 36) % 360;
  game.rainbowTrail.colors = Array.from({ length: 6 }, (_, i) => {
    return `hsl(${(baseHue + i * 20) % 360}, 100%, 50%)`;
  });
}
