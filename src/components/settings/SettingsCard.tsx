import { type ReactNode } from 'react';
import { motion } from 'framer-motion';

interface SettingsCardProps {
  title: string;
  icon: string;
  children: ReactNode;
}

/** Card reutilizável para seções de configuração */
export function SettingsCard({ title, icon, children }: SettingsCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.25 }}
      className="bg-[#111827] border border-gray-800 rounded-xl p-4 sm:p-5"
    >
      <h2 className="text-sm font-semibold text-gray-300 mb-3 flex items-center gap-2">
        <span>{icon}</span>
        <span>{title}</span>
      </h2>
      {children}
    </motion.div>
  );
}
