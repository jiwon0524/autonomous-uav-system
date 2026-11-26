import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';
export function Panel({title,icon:Icon,children,className='',extra}: {title:string;icon:LucideIcon;children:ReactNode;className?:string;extra?:ReactNode}) {
  return <section className={`panel ${className}`}><header className="panel-heading"><h2><Icon size={15}/>{title}</h2>{extra}</header>{children}</section>;
}
export function Badge({children,tone=''}: {children:ReactNode;tone?:string}) {return <span className={`badge ${tone}`}>{children}</span>;}
