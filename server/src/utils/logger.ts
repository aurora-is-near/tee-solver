import winston from 'winston';

const { combine, timestamp, printf, align, colorize } = winston.format;

export const logger = winston.createLogger({
  level: 'info',
  transports: [new winston.transports.Console()],
  format: combine(
    colorize({ all: true }),
    timestamp({
      format: 'YYYY-MM-DD HH:mm:ss.SSS',
    }),
    align(),
    printf((info) => `[${info.timestamp}] ${info.level}: ${info.message}`),
  ),
});
